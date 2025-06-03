# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("wasmtime/wasmtime_rb") do |ext|
  ext.extra_cargo_args += ["--crate-type", "cdylib"]
  ext.extra_cargo_args += ["--package", "wasmtime-rb"]
  ext.extra_rustflags = ["--cfg=rustix_use_libc"]
end
