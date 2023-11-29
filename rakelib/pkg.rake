CLOBBER.include("pkg/**/*.gem")
CLEAN.include("tmp/pkg")
CLEAN.include("tmp/pkg")

def gem_install_test(dotgem)
  dotgem = File.expand_path(File.join("..", dotgem), __dir__)
  tmpdir = File.expand_path("../tmp/pkg-test-#{Time.now.to_i}", __dir__)
  sh "gem install --verbose --install-dir #{tmpdir} #{dotgem}"

  wrapper = if defined?(Bundler)
    ->(&blk) { Bundler.with_unbundled_env { blk.call } }
  else
    ->(&blk) { blk.call }
  end

  testrun = ->(cmd) do
    cmd = cmd.chomp

    wrapper.call do
      old = ENV["GEM_HOME"]
      ENV["GEM_HOME"] = tmpdir
      ruby "-rwasmtime -e '(#{cmd}) || abort'"
      puts "✅ Passed (#{cmd})"
    rescue
      abort "❌ Failed (#{cmd})"
    ensure
      ENV["GEM_HOME"] = old
    end
  end

  testrun.call <<~RUBY
    Wasmtime::VERSION == "#{GEMSPEC.version}"
  RUBY

  testrun.call <<~RUBY
    Wasmtime::Engine.new.precompile_module("(module)").include?("ELF")
  RUBY

  FileUtils.rm_rf(tmpdir)
end

namespace :pkg do
  directory "pkg"

  desc "Build the source gem (#{GEMSPEC.name}-#{GEMSPEC.version}.gem)"
  task ruby: "pkg" do
    slug = "#{GEMSPEC.name}-#{GEMSPEC.version}"
    output_gempath = File.expand_path("../pkg/#{slug}.gem", __dir__)
    gemspec_path = "wasmtime.gemspec"
    base_dir = File.join("tmp/pkg", slug)
    staging_dir = File.join(base_dir, "stage")
    unpacked_dir = File.join(base_dir, "unpacked")
    vendor_dir = "ext/cargo-vendor" # this file gets cleaned up during gem install
    staging_gem_path = File.join(staging_dir, "#{slug}.gem")

    puts "Building source gem..."

    rm(output_gempath) if File.exist?(output_gempath)
    rm_rf(staging_dir)
    rm_rf(unpacked_dir)
    mkdir_p(staging_dir)
    cp(gemspec_path, staging_dir)

    GEMSPEC.files.each do |file|
      dest = File.join(staging_dir, file)
      mkdir_p(File.dirname(dest))
      cp(file, dest) if File.file?(file)
    end

    Dir.chdir(staging_dir) do
      cargo_config_path = ".cargo/config"
      final_gemspec = Gem::Specification.load(File.basename(gemspec_path))

      puts "Vendoring cargo dependencies to #{cargo_config_path}..."
      mkdir_p ".cargo"
      sh "cargo vendor --versioned-dirs --locked #{vendor_dir} >> #{cargo_config_path} 2>/dev/null"

      vendor_files = dirglob("./#{vendor_dir}/**/*").reject { |f| File.directory?(f) }
      # Ensure that all vendor files have the right read permissions,
      # which are needed to build the gem.
      # The permissions that we want _at least_ is readable by all for example `.rw-r--r--`
      vendor_files.each { |f| FileUtils.chmod("a+r", f) }
      final_gemspec.files += vendor_files
      final_gemspec.files += dirglob("**/.cargo/**/*").reject { |f| File.directory?(f) }

      puts "Building gem to #{unpacked_dir}.gem..."
      Gem::Package.build(final_gemspec, false, true, "#{slug}.gem")
    end

    puts "Unpacking gem to #{unpacked_dir}..."
    sh "gem unpack #{staging_gem_path} --target #{File.dirname(unpacked_dir)} --quiet"
    mv File.join(base_dir, slug), unpacked_dir

    puts "Verifying cargo dependencies are vendored..."
    sh "cargo verify-project --manifest-path #{File.join(unpacked_dir, "Cargo.toml")}"

    cp staging_gem_path, output_gempath

    puts <<~STATS
      \n\e[1m==== Source gem stats (#{File.basename(output_gempath)}) ====\e[0m
      - Path: #{output_gempath.delete_prefix(Dir.pwd + "/")}
      - Number of files: #{dirglob("#{unpacked_dir}/**/*").count}
      - Number of vendored deps: #{dirglob("#{unpacked_dir}/#{vendor_dir}/*").count}
      - Size (packed): #{filesize(output_gempath)} MB
      - Size (unpacked): #{filesize(*dirglob("#{unpacked_dir}/**/*"))} MB
    STATS
  end

  desc "Test source gem installation"
  task "ruby:test" => "pkg:ruby" do
    gem_install_test("pkg/#{GEMSPEC.name}-#{GEMSPEC.version}.gem")
  end

  ["x86_64-darwin", "arm64-darwin", "x86_64-linux"].each do |platform|
    desc "Test #{platform} gem installation"
    task "#{platform}:test" do
      gem_install_test("pkg/#{GEMSPEC.name}-#{GEMSPEC.version}-#{platform}.gem")
    end
  end
end
