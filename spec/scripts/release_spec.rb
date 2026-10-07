# frozen_string_literal: true

require_relative "../../scripts/release"

RSpec.describe Release do
  let(:tags) { %w[v0.3.0 v36.0.0 v46.0.0 v47.0.3 v48.0.1] }

  describe ".next_version" do
    def next_version(wasmtime, current, requested = nil, extra_tags: [])
      Release.next_version(wasmtime_version: wasmtime, current_version: current, tags: tags + extra_tags, requested: requested)
    end

    it "uses the wasmtime version when it's new" do
      expect(next_version("49.0.2", "48.0.1")).to eq("49.0.2")
    end

    it "uses the next patch version when the wasmtime version is released" do
      expect(next_version("48.0.1", "48.0.1")).to eq("48.0.2")
      expect(next_version("49.0.2", "49.0.2", extra_tags: %w[v49.0.2])).to eq("49.0.3")
    end

    it "skips patch versions that are already released" do
      expect(next_version("48.0.2", "48.0.2", extra_tags: %w[v48.0.2])).to eq("48.0.3")
    end

    it "catches up with wasmtime when it's ahead of the gem" do
      expect(next_version("48.0.5", "48.0.2", extra_tags: %w[v48.0.2])).to eq("48.0.5")
    end

    it "compares versions numerically" do
      expect(next_version("49.0.9", "49.0.9", extra_tags: %w[v49.0.9])).to eq("49.0.10")
    end

    it "accepts a valid requested version" do
      expect(next_version("49.0.2", "48.0.1", "49.0.2")).to eq("49.0.2")
    end

    it "treats an empty requested version as not given" do
      expect(next_version("49.0.2", "48.0.1", "")).to eq("49.0.2")
    end

    it "rejects a requested version that doesn't match wasmtime" do
      expect { next_version("49.0.2", "48.0.1", "50.0.0") }.to raise_error(Release::Error, /major and minor/)
    end

    it "rejects a requested version that is already released" do
      expect { next_version("48.0.1", "47.0.3", "48.0.1") }.to raise_error(Release::Error, /already released/)
    end

    it "rejects a requested version that isn't greater than the current one" do
      expect { next_version("48.0.1", "48.0.1", "48.0.0") }.to raise_error(Release::Error, /isn't greater/)
    end

    it "rejects malformed versions" do
      expect { next_version("49.0.2", "48.0.1", "49.0") }.to raise_error(Release::Error, /MAJOR.MINOR.PATCH/)
      expect { next_version("49.0.2-rc.1", "48.0.1") }.to raise_error(Release::Error, /MAJOR.MINOR.PATCH/)
    end
  end

  describe ".previous_tag" do
    it "returns the highest tag below the given one" do
      expect(Release.previous_tag("v49.0.2", tags)).to eq("v48.0.1")
      expect(Release.previous_tag("v48.0.2", tags + %w[v49.0.2])).to eq("v48.0.1")
      expect(Release.previous_tag("v49.0.10", tags + %w[v49.0.2 v49.0.9 v49.0.10])).to eq("v49.0.9")
    end

    it "returns nil for the lowest tag" do
      expect(Release.previous_tag("v0.3.0", tags)).to be_nil
    end
  end

  describe ".wasmtime_version" do
    it "reads the wasmtime package version" do
      cargo_lock = <<~TOML
        [[package]]
        name = "wasmtime-environ"
        version = "48.0.0"

        [[package]]
        name = "wasmtime"
        version = "48.0.1"
      TOML
      expect(Release.wasmtime_version(cargo_lock)).to eq("48.0.1")
    end

    it "reads the repository's Cargo.lock" do
      cargo_lock = File.read(File.expand_path("../../Cargo.lock", __dir__))
      expect(Release.wasmtime_version(cargo_lock)).to match(Release::SEMVER)
    end
  end

  describe ".bump_version_rb" do
    it "replaces the version" do
      version_rb = File.read(File.expand_path("../../lib/wasmtime/version.rb", __dir__))
      bumped = Release.bump_version_rb(version_rb, "99.0.0")
      expect(Release.current_version(bumped)).to eq("99.0.0")
    end
  end

  describe ".bump_gemfile_lock" do
    it "replaces only the gem's own entry" do
      gemfile_lock = File.read(File.expand_path("../../Gemfile.lock", __dir__))
      bumped = Release.bump_gemfile_lock(gemfile_lock, "99.0.0")
      expect(bumped).to include("    wasmtime (99.0.0)\n")
      expect(bumped.lines - gemfile_lock.lines).to eq(["    wasmtime (99.0.0)\n"])
    end

    it "fails when the entry isn't found" do
      expect { Release.bump_gemfile_lock("GEM\n", "99.0.0") }.to raise_error(Release::Error, /found 0/)
    end
  end

  describe ".add_changelog_entry" do
    let(:changelog) do
      <<~MARKDOWN
        # Changelog

        ## [v48.0.1](https://example.com/tree/v48.0.1) (2026-09-03)

        Notes.
      MARKDOWN
    end

    it "adds an entry to fill in above the newest one" do
      updated = Release.add_changelog_entry(changelog, version: "48.0.2", previous_tag: "v48.0.1", repo_url: "https://example.com", date: "2026-10-07")
      expect(updated).to eq(<<~MARKDOWN)
        # Changelog

        ## [v48.0.2](https://example.com/tree/v48.0.2) (2026-10-07)

        [Full Changelog](https://example.com/compare/v48.0.1...v48.0.2)

        #{Release::CHANGELOG_PLACEHOLDER}

        ## [v48.0.1](https://example.com/tree/v48.0.1) (2026-09-03)

        Notes.
      MARKDOWN
    end

    it "makes the changelog fail the release check until it's filled in" do
      updated = Release.add_changelog_entry(changelog, version: "48.0.2", previous_tag: "v48.0.1", repo_url: "https://example.com", date: "2026-10-07")
      expect(Release.changelog_errors(updated, "v48.0.2")).to eq(["Fill in the release notes in CHANGELOG.md"])

      filled_in = updated.sub(Release::CHANGELOG_PLACEHOLDER, "- Fix a bug")
      expect(Release.changelog_errors(filled_in, "v48.0.2")).to be_empty
    end
  end

  describe ".changelog_errors" do
    it "requires an entry for the tag" do
      expect(Release.changelog_errors("# Changelog\n", "v48.0.2")).to eq(["CHANGELOG.md has no entry for v48.0.2"])
    end
  end

  describe ".add_excluded_tag" do
    it "creates the config" do
      expect(Release.add_excluded_tag(nil, "v48.0.2")).to eq("exclude-tags=v48.0.2\n")
    end

    it "appends to existing exclude-tags" do
      config = "since-tag=v1.0.0\nexclude-tags=v47.0.4\n"
      expect(Release.add_excluded_tag(config, "v48.0.2")).to eq("since-tag=v1.0.0\nexclude-tags=v47.0.4,v48.0.2\n")
    end

    it "adds exclude-tags to a config without it" do
      expect(Release.add_excluded_tag("since-tag=v1.0.0", "v48.0.2")).to eq("since-tag=v1.0.0\nexclude-tags=v48.0.2\n")
    end

    it "doesn't add a tag twice" do
      expect(Release.add_excluded_tag("exclude-tags=v48.0.2\n", "v48.0.2")).to eq("exclude-tags=v48.0.2\n")
    end
  end

  describe ".missing_excluded_tags" do
    it "ignores excluded and historical tags" do
      expect(Release.missing_excluded_tags(%w[v0.3.0 v48.0.2 v48.0.3], "exclude-tags=v48.0.2\n")).to eq(%w[v48.0.3])
      expect(Release.missing_excluded_tags(%w[v0.3.0], nil)).to be_empty
    end
  end
end
