# frozen_string_literal: true

source "https://rubygems.org"

# Specify your gem's dependencies in wasmtime.gemspec
gemspec

group :development do
  gem "rake", "~> 13.2"
  gem "rake-compiler"
  gem "standard", "~> 1.40"
  gem "get_process_mem"
  gem "yard", require: false
  gem "yard-rustdoc", "~> 0.3.2", require: false
  gem "benchmark-ips", require: false
end

group :test do
  gem "rspec", "~> 3.13"
end
