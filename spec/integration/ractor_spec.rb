if RUBY_VERSION.start_with?("3.")
  RSpec.describe "Ractor" do
    let(:wat) { <<~WAT }
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

    it "supports running inside Ractors" do
      r = Ractor.new(wat) do |wat|
        engine = Wasmtime::Engine.new
        mod = Wasmtime::Module.new(engine, wat)
        store_data = Object.new
        store = Wasmtime::Store.new(engine, store_data)
        Wasmtime::Instance.new(store, mod).invoke("hello")
      end

      result = r.take
      expect(result).to eq([1, 2, 3.0, 4.0])
    end

    it "supports sharing Engine & Module with Ractors" do
      engine = Wasmtime::Engine.new
      mod = Wasmtime::Module.new(engine, wat)

      Ractor.make_shareable(engine)
      Ractor.make_shareable(mod)

      ractors = []
      3.times do
        ractors << Ractor.new(engine, mod) do |engine, mod|
          store_data = Object.new
          store = Wasmtime::Store.new(engine, store_data)
          Wasmtime::Instance.new(store, mod).invoke("hello")
        end
      end

      ractors.each do |ractor|
        expect(ractor.take).to eq([1, 2, 3.0, 4.0])
      end
    end
  end
end
