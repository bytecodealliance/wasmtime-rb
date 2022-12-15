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
      import0 = Wasmtime::Func.new(store, [:externref], [:externref]) { |o| o }
      import1 = Wasmtime::Func.new(store, [], []) { raise SomeError }
      instance = Wasmtime::Instance.new(store, mod, [import0, import1])
      instance.invoke("hello")
      instance.invoke("f0", BasicObject.new)
      begin
        instance.invoke("f1")
      rescue SomeError # no-op
      end
    end

    wasmtime_interaction.call # warm-up
    GC.start

    before = GetProcessMem.new.kb
    iterations = (ENV["TIMES"] || 500_000).to_i
    gc_every = (ENV["GC_EVERY"] || iterations / 100).to_i
    iterations.to_i.times do |i|
      wasmtime_interaction.call
      if i % gc_every == 0
        GC.start
        after = GetProcessMem.new.kb
        if before != after
          puts format("Mem change: %d KiB -> %d KiB (%+d), i=#{i}", before, after, after - before)
        end
        before = after
      end
    end
  end

  if RbConfig::CONFIG["host_os"] == "linux"
    begin
      require "ruby_memcheck"
      require "ruby_memcheck/rspec/rake_task"

      RubyMemcheck.config(binary_name: "ext")

      RubyMemcheck::RSpec::RakeTask.new(check: "compile:dev")
    rescue LoadError
      task :check do
        abort 'Please add `gem "ruby_memcheck"` to your Gemfile to use the "mem:check" task'
      end
    end
  else
    task :check do
      abort 'The "mem:check" task is only available on Linux'
    end
  end
end
