# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("wasmtime/wasmtime_rb") do |ext|
  ext.extra_cargo_args += ["--crate-type", "cdylib"]
  ext.extra_cargo_args += ["--package", "wasmtime-rb"]
  ext.extra_rustflags = ["--cfg=rustix_use_libc"]

  # MinGW PE/COFF native gems fail to load on Windows when packaged with the
  # embedded DWARF debuginfo produced by the release profile. Disable debuginfo
  # for Windows GNU/UCRT builds while preserving it for other platforms.
  if RbConfig::CONFIG.fetch("host_os").match?(/mingw|mswin/)
    ext.extra_rustflags += ["-C", "debuginfo=0"]
  end
end
