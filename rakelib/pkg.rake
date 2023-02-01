CLOBBER.include("pkg/**/*.gem")
CLEAN.include("tmp/pkg")
CLEAN.include("tmp/pkg")

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

      final_gemspec.files += dirglob("**/.cargo/**/*")
      final_gemspec.files += dirglob("./#{vendor_dir}/**/*")
      final_gemspec.files.reject! { |f| File.directory?(f) }

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

  desc "Test source gem installation (#{GEMSPEC.name}-#{GEMSPEC.version}.gem)"
  task "ruby:test" => "pkg:ruby" do
    sh "gem install --verbose --install-dir tmp/source-gem-test pkg/#{GEMSPEC.name}-#{GEMSPEC.version}.gem"

    wrapper = if defined?(Bundler)
      ->(&blk) { Bundler.with_unbundled_env { blk.call } }
    else
      ->(&blk) { blk.call }
    end

    testrun = ->(cmd) do
      cmd = cmd.chomp

      wrapper.call do
        old = ENV["GEM_HOME"]
        ENV["GEM_HOME"] = "tmp/source-gem-test"
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

    FileUtils.rm_rf("tmp/source-gem-test")
  end
end
