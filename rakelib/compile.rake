require "rb_sys/extensiontask"

RbSys::ExtensionTask.new("wasmtime-rb", GEMSPEC) do |ext|
  ext.lib_dir = "lib/wasmtime"
end

namespace :compile do
  desc 'Compile the extension in "release" mode'
  task release: ["env:release", "compile"]

  desc 'Compile the extension in "dev" mode'
  task dev: ["env:dev", "compile"]
end
