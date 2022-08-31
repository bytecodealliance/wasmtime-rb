RSpec.describe "Hello World" do
  it "properly converts return args (i32, i64, f32, f64)" do
    instance = compile <<~WAT
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

    result = instance.invoke("hello", [])

    expect(result).to eq([1, 2, 3.0, 4.0])
  end

  it "can accept basic args" do
    instance = compile <<~WAT
      (module
        (func $module/add_three (param $0 i32) (param $1 i64) (param $2 f32) (param $3 f64) (result i32 i64 f32 f64)
          local.get $0
          i32.const 3
          i32.add

          local.get $1
          i64.const 3
          i64.add

          local.get $2
          f32.const 3.0
          f32.add

          local.get $3
          f64.const 3.0
          f64.add
        )
        (export "add_three" (func $module/add_three))
      )
    WAT
    result = instance.invoke("add_three", [1, 2, 3.0, 4.0])

    expect(result).to eq([4, 5, 6.0, 7.0])
  end

  it "has exports" do
    instance = compile <<~WAT
      (module
        (func $module/hello (result i32)
          i32.const 1
        )
        (export "hello" (func $module/hello))
      )
    WAT

    result = instance.exports.transform_values(&:type_name)

    expect(result).to eq({hello: :func})
  end

  def compile(wat)
    data = {}
    config = Wasmtime::Config.new
    engine = Wasmtime::Engine.new(config)
    store = Wasmtime::Store.new engine, data
    mod = Wasmtime::Module.new engine, wat
    Wasmtime::Instance.new(store, mod)
  end
end
