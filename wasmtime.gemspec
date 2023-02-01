# frozen_string_literal: true

require_relative "lib/wasmtime/version"

Gem::Specification.new do |spec|
  spec.name = "wasmtime"
  spec.version = Wasmtime::VERSION
  spec.authors = ["The Wasmtime Project Developers"]
  spec.email = ["hello@bytecodealliance.org"]

  spec.summary = "Wasmtime bindings for Ruby"
  spec.description = "A Ruby binding for Wasmtime, a WebAssembly runtime."
  spec.homepage = "https://github.com/BytecodeAlliance/wasmtime-rb"
  spec.license = "Apache-2.0"
  spec.required_ruby_version = ">= 2.7.0"

  spec.metadata["homepage_uri"] = "https://github.com/BytecodeAlliance/wasmtime-rb"
  spec.metadata["source_code_uri"] = "https://github.com/BytecodeAlliance/wasmtime-rb"
  spec.metadata["cargo_crate_name"] = "wasmtime-rb"
  spec.metadata["changelog_uri"] = "https://github.com/bytecodealliance/wasmtime-rb/blob/main/CHANGELOG.md"

  spec.files = Dir["{lib,ext}/**/*", "LICENSE", "README.md", "Cargo.*"]
  spec.files.reject! { |f| File.directory?(f) }
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  spec.extensions = ["ext/extconf.rb"] # Future: ["ext/Cargo.toml"] with rubygems >= 3.3.24

  spec.rdoc_options += ["--exclude", "vendor"]

  # Can be removed for binary gems and rubygems >= 3.3.24
  spec.add_dependency "rb_sys", "~> 0.9.59"
end
