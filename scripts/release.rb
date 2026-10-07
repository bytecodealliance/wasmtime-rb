#!/usr/bin/env ruby
# frozen_string_literal: true

# Helpers for the release workflows (.github/workflows/release-*.yml). Only
# uses the standard library, so it runs without `bundle install`.
#
# Commands run in the root of the branch being released:
#
#   release.rb bump [VERSION]          Updates version.rb and Gemfile.lock to
#                                      VERSION, or the next version (see
#                                      next_version), and prints version=,
#                                      previous=, wasmtime=
#   release.rb changelog-entry VERSION PREVIOUS_TAG REPO_URL
#                                      Adds a changelog entry to fill in
#   release.rb exclude-tag TAG         Adds TAG to exclude-tags
#   release.rb missing-excluded-tags   Prints tags not on HEAD nor excluded
#   release.rb check-changelog TAG     Fails unless TAG has a filled-in entry
module Release
  class Error < StandardError; end

  VERSION_RB = "lib/wasmtime/version.rb"
  GEMFILE_LOCK = "Gemfile.lock"
  CARGO_LOCK = "Cargo.lock"
  CHANGELOG = "CHANGELOG.md"
  CHANGELOG_CONFIG = ".github_changelog_generator"
  CHANGELOG_PLACEHOLDER = "<!-- RELEASE NOTES PLACEHOLDER: describe the changes in this release and remove this line. -->"
  # Tags that aren't on `main` but predate release branches.
  HISTORICAL_TAGS_OFF_MAIN = %w[v0.3.0].freeze

  SEMVER = /\A\d+\.\d+\.\d+\z/

  module_function

  # The version for the next release. The gem version tracks the `wasmtime`
  # crate version; when that version is already released (e.g. a security
  # release without an upstream release), the next unused patch version is
  # used instead. See CONTRIBUTING.md#versioning.
  def next_version(wasmtime_version:, current_version:, tags:, requested: nil)
    [wasmtime_version, current_version].each { |v| check_semver(v) }
    released = tags

    version =
      if requested.nil? || requested.empty?
        candidate = [Gem::Version.new(wasmtime_version), Gem::Version.new(current_version)].max.to_s
        candidate = bump_patch(candidate) while released.include?("v#{candidate}") || !greater?(candidate, current_version)
        candidate
      else
        requested
      end

    check_semver(version)
    unless minor_line(version) == minor_line(wasmtime_version)
      raise Error, "#{version} doesn't match the major and minor version of wasmtime #{wasmtime_version}"
    end
    raise Error, "v#{version} is already released" if released.include?("v#{version}")
    raise Error, "#{version} isn't greater than the current version #{current_version}" unless greater?(version, current_version)

    version
  end

  # The highest tag below `tag`, or nil.
  def previous_tag(tag, tags)
    target = Gem::Version.new(tag.delete_prefix("v"))
    tags
      .select { |t| t.match?(/\Av\d+\.\d+\.\d+\z/) }
      .map { |t| Gem::Version.new(t.delete_prefix("v")) }
      .select { |v| v < target }
      .max
      &.then { |v| "v#{v}" }
  end

  def wasmtime_version(cargo_lock)
    cargo_lock[/^name = "wasmtime"\nversion = "([^"]+)"$/, 1] or raise Error, "no wasmtime package in #{CARGO_LOCK}"
  end

  def current_version(version_rb)
    version_rb[/^  VERSION = "([^"]+)"$/, 1] or raise Error, "no VERSION in #{VERSION_RB}"
  end

  def bump_version_rb(version_rb, version)
    replace_one(version_rb, /^  VERSION = "[^"]+"$/, %(  VERSION = "#{version}"), VERSION_RB)
  end

  # Only the gem's own entry, under the PATH source's specs.
  def bump_gemfile_lock(gemfile_lock, version)
    replace_one(gemfile_lock, /^    wasmtime \([^)]+\)$/, "    wasmtime (#{version})", GEMFILE_LOCK)
  end

  # Adds an entry for `version` above the newest one, for the maintainer to
  # fill in. Used on release branches, where the changelog generator doesn't
  # work.
  def add_changelog_entry(changelog, version:, previous_tag:, repo_url:, date:)
    index = changelog.index(/^## /) or raise Error, "no entries in #{CHANGELOG}"
    entry = <<~MARKDOWN
      ## [v#{version}](#{repo_url}/tree/v#{version}) (#{date})

      [Full Changelog](#{repo_url}/compare/#{previous_tag}...v#{version})

      #{CHANGELOG_PLACEHOLDER}

    MARKDOWN
    changelog.dup.insert(index, entry)
  end

  # Errors that prevent releasing `tag` with `changelog`.
  def changelog_errors(changelog, tag)
    errors = []
    errors << "#{CHANGELOG} has no entry for #{tag}" unless changelog.match?(/^## \[#{Regexp.escape(tag)}\]/)
    errors << "Fill in the release notes in #{CHANGELOG}" if changelog.include?(CHANGELOG_PLACEHOLDER)
    errors
  end

  # `config` is the content of the changelog generator config, or nil.
  def excluded_tags(config)
    config.to_s[/^exclude-tags=(.*)$/, 1].to_s.split(",").map(&:strip).reject(&:empty?)
  end

  def add_excluded_tag(config, tag)
    config = config.to_s
    tags = excluded_tags(config)
    return config if tags.include?(tag)

    line = "exclude-tags=#{(tags + [tag]).join(",")}"
    if config.match?(/^exclude-tags=/)
      config.sub(/^exclude-tags=.*$/, line)
    else
      config += "\n" unless config.empty? || config.end_with?("\n")
      "#{config}#{line}\n"
    end
  end

  # Tags that aren't on `main` and aren't excluded from its changelog.
  def missing_excluded_tags(tags_not_on_main, config)
    tags_not_on_main - HISTORICAL_TAGS_OFF_MAIN - excluded_tags(config)
  end

  def check_semver(version)
    raise Error, "'#{version}' is not a MAJOR.MINOR.PATCH version" unless version.match?(SEMVER)
  end

  def greater?(a, b)
    Gem::Version.new(a) > Gem::Version.new(b)
  end

  def bump_patch(version)
    major, minor, patch = version.split(".").map(&:to_i)
    "#{major}.#{minor}.#{patch + 1}"
  end

  def minor_line(version)
    version.split(".").first(2)
  end

  def replace_one(text, pattern, replacement, file)
    count = text.scan(pattern).size
    raise Error, "expected one match for #{pattern.inspect} in #{file}, found #{count}" unless count == 1

    text.sub(pattern, replacement)
  end

  module CLI
    module_function

    def run(command, *args)
      case command
      when "bump"
        tags = git("tag", "-l", "v*")
        wasmtime_version = Release.wasmtime_version(File.read(CARGO_LOCK))
        version = Release.next_version(
          wasmtime_version: wasmtime_version,
          current_version: Release.current_version(File.read(VERSION_RB)),
          tags: tags,
          requested: args[0]
        )
        edit(VERSION_RB) { |text| Release.bump_version_rb(text, version) }
        edit(GEMFILE_LOCK) { |text| Release.bump_gemfile_lock(text, version) }
        puts "version=#{version}"
        puts "previous=#{Release.previous_tag("v#{version}", tags)}"
        puts "wasmtime=#{wasmtime_version}"
      when "current-version"
        puts Release.current_version(File.read(VERSION_RB))
      when "changelog-entry"
        version, previous_tag, repo_url = args.fetch(0), args.fetch(1), args.fetch(2)
        edit(CHANGELOG) do |text|
          Release.add_changelog_entry(text, version: version, previous_tag: previous_tag, repo_url: repo_url, date: Time.now.utc.strftime("%F"))
        end
      when "exclude-tag"
        tag = args.fetch(0)
        config = File.exist?(CHANGELOG_CONFIG) ? File.read(CHANGELOG_CONFIG) : nil
        File.write(CHANGELOG_CONFIG, Release.add_excluded_tag(config, tag))
      when "missing-excluded-tags"
        config = File.exist?(CHANGELOG_CONFIG) ? File.read(CHANGELOG_CONFIG) : nil
        puts Release.missing_excluded_tags(git("tag", "--no-merged", "HEAD", "v*"), config)
      when "check-changelog"
        errors = Release.changelog_errors(File.read(CHANGELOG), args.fetch(0))
        raise Error, errors.join("; ") unless errors.empty?
      else
        raise Error, "unknown command #{command.inspect}; see #{__FILE__}"
      end
    rescue IndexError
      raise Error, "missing arguments for #{command}; see #{__FILE__}"
    end

    def edit(file)
      File.write(file, yield(File.read(file)))
    end

    def git(*args)
      output = IO.popen(["git", *args], &:read)
      raise Error, "git #{args.join(" ")} failed" unless $?.success?

      output.split("\n")
    end
  end
end

if $PROGRAM_NAME == __FILE__
  begin
    Release::CLI.run(*ARGV)
  rescue Release::Error => e
    warn "#{ENV["GITHUB_ACTIONS"] ? "::error::" : "error: "}#{e.message}"
    exit 1
  end
end
