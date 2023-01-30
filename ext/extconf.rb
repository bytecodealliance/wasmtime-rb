# frozen_string_literal: true

require "mkmf"
require "rb_sys/mkmf"

create_rust_makefile("wasmtime/wasmtime_rb")

contents = File.read("Makefile")
contents.gsub!("$(CARGO) rustc", "$(CARGO) rustc --crate-type cdylib")
File.write("Makefile", contents)
