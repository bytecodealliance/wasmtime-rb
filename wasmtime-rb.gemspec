# frozen_string_literal: true

require_relative "lib/wasmtime/version"

Gem::Specification.new do |spec|
  spec.name = "wasmtime-rb"
  spec.version = Wasmtime::VERSION
  spec.authors = ["Ian Ker-Seymer"]
  spec.email = ["hello@ianks.com"]

  spec.summary = "Wasmtime bindings for Ruby"
  spec.description = "A Ruby binding for Wasmtime, a WebAssembly runtime."
  spec.homepage = "https://github.com/BytecodeAlliance/wasmtime-rb"
  spec.license = "Apache-2.0"
  spec.required_ruby_version = ">= 2.7.0"

  spec.metadata["homepage_uri"] = "https://github.com/BytecodeAlliance/wasmtime-rb"
  spec.metadata["source_code_uri"] = "https://github.com/BytecodeAlliance/wasmtime-rb"
  # spec.metadata["changelog_uri"] = "TODO: Put your gem's CHANGELOG.md URL here."

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir.chdir(__dir__) do
    `git ls-files -z`.split("\x0").reject do |f|
      (f == __FILE__) || f.match(%r{\A(?:(?:bin|test|spec|features)/|\.(?:git|travis|circleci)|appveyor)})
    end
  end
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/wasmtime_rb/Cargo.toml"]

  # Uncomment to register a new dependency of your gem
  # spec.add_dependency "example-gem", "~> 1.0"

  # For more information and examples about making a new gem, check out our
  # guide at: https://bundler.io/guides/creating_gem.html
end
