require "rb_sys/extensiontask"

RbSys::ExtensionTask.new("wasmtime-rb", GEMSPEC) do |ext|
  ext.lib_dir = "lib/wasmtime"
end
