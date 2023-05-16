# frozen_string_literal: true

source "https://rubygems.org"

# Specify your gem's dependencies in wasmtime.gemspec
gemspec

group :development do
  gem "rake", "~> 13.0"
  gem "rake-compiler"
  gem "rb_sys", "~> 0.9.65"
  gem "standard", "~> 1.28"
  gem "get_process_mem"
  gem "ruby-lsp", require: false
  gem "yard", require: false
  gem "yard-rustdoc", "~> 0.3.2", require: false
  gem "benchmark-ips", require: false
end

group :test do
  gem "rspec", "~> 3.1"
end
