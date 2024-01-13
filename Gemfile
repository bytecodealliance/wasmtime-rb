# frozen_string_literal: true

source "https://rubygems.org"

# Specify your gem's dependencies in wasmtime.gemspec
gemspec

group :development do
  gem "rake", "~> 13.1"
  gem "rake-compiler"
  gem "standard", "~> 1.32"
  gem "get_process_mem"
  gem "yard", require: false
  gem "yard-rustdoc", "~> 0.3.2", require: false
  gem "benchmark-ips", require: false
end

group :test do
  gem "rspec", "~> 3.1"
end

group :development, :test do
  gem "pry", "~> 0.14.2"
  gem "solargraph", "~> 0.50.0"
  gem "rubocop", "~> 1.59"
end
