namespace :mem do
  desc "Runs a WebAssembly function ENV['TIMES'] in a loop looking for memory leaks. "
  task :growth do
    require "wasmtime"
    require "get_process_mem"

    precompiled = Wasmtime::Engine.new.precompile_module(<<~WAT)
      (module
        (func $module/hello (result i32 i64 f32 f64)
          i32.const 1
          i64.const 2
          f32.const 3.0
          f64.const 4.0
        )

        (export "hello" (func $module/hello))
      )
    WAT

    wasmtime_interaction = -> do
      config = Wasmtime::Config.new
      engine = Wasmtime::Engine.new(config)
      store = Wasmtime::Store.new(engine, {})
      mod = Wasmtime::Module.deserialize(engine, precompiled)
      instance = Wasmtime::Instance.new(store, mod)
      instance.invoke("hello", [])
      GC.start
    end

    wasmtime_interaction.call # warm-up

    (ENV["TIMES"] || 5_000).to_i.times do |i|
      before = GetProcessMem.new.kb
      wasmtime_interaction.call
      after = GetProcessMem.new.kb
      if before != after
        puts format("Mem change: %d KiB -> %d KiB (%+d), i=#{i}", before, after, after - before)
      end
    end
  end
end
