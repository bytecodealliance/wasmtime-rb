class SomeError < StandardError; end

namespace :mem do
  desc "Runs a WebAssembly function ENV['TIMES'] in a loop looking for memory leaks. "
  task :growth do
    require "wasmtime"
    require "get_process_mem"

    precompiled = Wasmtime::Engine.new.precompile_module(<<~WAT)
      (module
        (import "" "" (func (param externref) (result externref)))
        (import "" "" (func))
        (func $module/hello (result i32 i64 f32 f64)
          i32.const 1
          i64.const 2
          f32.const 3.0
          f64.const 4.0
        )

        (export "hello" (func $module/hello))
        (export "f0" (func 0))
        (export "f1" (func 1))
      )
    WAT

    wasmtime_interaction = -> do
      config = Wasmtime::Config.new
      engine = Wasmtime::Engine.new(config)
      store = Wasmtime::Store.new(engine, {})
      mod = Wasmtime::Module.deserialize(engine, precompiled)
      import0 = Wasmtime::Func.new(store, Wasmtime::FuncType.new([:externref], [:externref]), false, ->(i) { i })
      import1 = Wasmtime::Func.new(store, Wasmtime::FuncType.new([], []), false, -> { raise SomeError })
      instance = Wasmtime::Instance.new(store, mod, [import0, import1])
      instance.invoke("hello", [])
      instance.invoke("f0", [BasicObject.new])
      begin
        instance.invoke("f1", [])
      rescue SomeError # no-op
      end
      GC.start
    end

    wasmtime_interaction.call # warm-up

    (ENV["TIMES"] || 10_000).to_i.times do |i|
      before = GetProcessMem.new.kb
      wasmtime_interaction.call
      after = GetProcessMem.new.kb
      if before != after
        puts format("Mem change: %d KiB -> %d KiB (%+d), i=#{i}", before, after, after - before)
      end
    end
  end
end
