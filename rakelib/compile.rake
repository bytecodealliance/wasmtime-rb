require "rb_sys/extensiontask"

RbSys::ExtensionTask.new("wasmtime-rb", GEMSPEC) do |ext|
  ext.lib_dir = "lib/wasmtime"
  ext.platform = ENV["RUBY_PLATFORM_TARGET"] if ENV["RUBY_PLATFORM_TARGET"]

  ext.cross_compiling do |spec|
    ruby_versions = ENV.fetch("STABLE_RUBY_VERSIONS", "").split(",").reject(&:empty?)
    next if ruby_versions.empty?

    ruby_api_version = ->(version) { version.split(".")[0, 2].join(".") }
    sorted_ruby_versions = ruby_versions.sort_by { |version| version.split(".").map(&:to_i) }
    min_ruby = ruby_api_version.call(sorted_ruby_versions.first)
    max_ruby = ruby_api_version.call(sorted_ruby_versions.last)

    spec.required_ruby_version = [">= #{min_ruby}", "< #{max_ruby.succ}.dev"]
  end
end
