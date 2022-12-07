require "rake/extensiontask"

CROSS_PLATFORMS = [ENV["RUBY_TARGET"]].compact
SOURCE_PATTERN = "**/src/**/*.{rs,toml,lock}"

Rake::ExtensionTask.new("wasmtime_rb", GEMSPEC) do |ext|
  ext.lib_dir = "lib/wasmtime"
  ext.ext_dir = "ext"
  ext.cross_compile = ENV.key?("RUST_TARGET")
  ext.cross_platform = CROSS_PLATFORMS

  ext.cross_compiling do |gem_spec|
    # No need for rb_sys to compile
    gem_spec.dependencies.reject! { |d| d.name == "rb_sys" }

    # Remove unnecessary files
    gem_spec.files -= Dir[SOURCE_PATTERN, "**/Cargo.*", "**/extconf.rb"]
  end
end

namespace :compile do
  desc 'Compile the extension in "release" mode'
  task release: ["env:release", "compile"]

  desc 'Compile the extension in "dev" mode'
  task dev: ["env:dev", "compile"]
end
