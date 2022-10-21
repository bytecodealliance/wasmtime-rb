require "rake/extensiontask"

GEMSPEC = Bundler.load_gemspec("wasmtime-rb.gemspec")

CROSS_PLATFORMS = [
  "aarch64-linux",
  "arm64-darwin",
  "x86_64-darwin",
  "x86_64-linux",
  "x86_64-linux-musl"
]

SOURCE_PATTERN = "**/src/**/*.{rs,toml,lock}"

Rake::ExtensionTask.new("ext", GEMSPEC) do |ext|
  ext.lib_dir = "lib/wasmtime"
  ext.ext_dir = "ext"
  ext.cross_compile = ENV.key?("RUST_TARGET")
  ext.cross_platform = CROSS_PLATFORMS

  ext.cross_compiling do |gem_spec|
    # No need for rb_sys to compile
    gem_spec.dependencies.reject! { |d| d.name == "rb_sys" }

    # Remove unnecessary files
    gem_spec.files -= Dir[SOURCE_PATTERN, "**/Cargo.*", "extconf.rb"]
  end
end
