require "spec_helper"

module Wasmtime
  module Component
    RSpec.describe Linker do
      let(:linker) { Linker.new(engine) }

      it "disallows linker reentrance" do
        linker.root do
          expect { linker.root }.to raise_error(Wasmtime::Error, /reentrant/)
        end
      end

      it "disallows linker instance reentrance" do
        linker.instance("foo") do |foo|
          foo.instance("bar") do |_|
            expect { foo.instance("bar") {} }.to raise_error(Wasmtime::Error, /reentrant/)
            expect { foo.module("bar", Module.new(engine, wat)) {} }.to raise_error(Wasmtime::Error, /reentrant/)
          end
        end
      end

      it "disallows using LinkerInstance outside its block" do
        leaked_instance = nil
        linker.root { |root| leaked_instance = root }
        expect { leaked_instance.instance("foo") {} }
          .to raise_error(Wasmtime::Error, /LinkerInstance went out of scope/)
      end

      describe "#instantiate" do
        it "returns a Component::Instance" do
          component = Component.new(engine, "(component)")
          store = Store.new(engine)
          expect(linker.instantiate(store, component))
            .to be_instance_of(Wasmtime::Component::Instance)
        end
      end

      describe "LinkerInstance#func_new" do
        let(:t) { Type }

        it "defines a function" do
          linker.root do |root|
            root.func_new("greet") do |name|
              t.string.wrap("Hello, #{name}!")
            end
          end

          expect(linker).to be_a(Linker)
        end

        it "defines functions in nested instances" do
          linker.instance("math") do |math|
            math.func_new("add") do |a, b|
              t.u32.wrap(a + b)
            end
          end

          expect(linker).to be_a(Linker)
        end

        it "requires a block" do
          expect {
            linker.root do |root|
              root.func_new("no-block")
            end
          }.to raise_error(ArgumentError, /no block given/)
        end
      end

      describe "LinkerInstance#func_new integration" do
        before(:all) do
          @host_imports_component = Component.from_file(
            GLOBAL_ENGINE,
            "spec/fixtures/host_func_imports.wasm"
          )
        end

        let(:t) { Type }

        # Helper to stub all required imports except the specified one(s)
        # @param except [Symbol, Array<Symbol>, nil] Import(s) to skip stubbing (nil = stub all)
        def stub_component_imports(linker, except: nil)
          skip_funcs = Array(except).map(&:to_s).to_set

          # Stub root functions
          linker.root do |root|
            root.func_new("greet") { |name| t.string.wrap(name) } unless skip_funcs.include?("greet")
            root.func_new("add") { |a, b| t.u32.wrap(a + b) } unless skip_funcs.include?("add")
            root.func_new("get-constant") { t.u32.wrap(0) } unless skip_funcs.include?("get-constant")
            unless skip_funcs.include?("make-point")
              point_type = t.record("x" => t.s32, "y" => t.s32)
              root.func_new("make-point") do |x, y|
                point_type.wrap({"x" => x, "y" => y})
              end
            end
            root.func_new("sum-list") { |nums| t.s32.wrap(nums.sum) } unless skip_funcs.include?("sum-list")
            root.func_new("maybe-double") { |n| t.option(t.u32).wrap(n) } unless skip_funcs.include?("maybe-double")
            unless skip_funcs.include?("safe-divide")
              root.func_new("safe-divide") do |a, b|
                t.result(t.u32, t.string).wrap(Result.ok(a))
              end
            end
            root.func_new("get-numbers") { t.list(t.s32).wrap([]) } unless skip_funcs.include?("get-numbers")
            unless skip_funcs.include?("make-tuple")
              tuple_type = t.tuple([t.u32, t.string, t.bool])
              root.func_new("make-tuple") do |n, s, b|
                tuple_type.wrap([n, s, b])
              end
            end
            unless skip_funcs.include?("analyze-numbers")
              tuple_type = t.tuple([t.s32, t.list(t.s32)])
              root.func_new("analyze-numbers") do |nums|
                tuple_type.wrap([0, nums])
              end
            end
            # Additional integer types
            root.func_new("echo-s8") { |n| t.s8.wrap(n) } unless skip_funcs.include?("echo-s8")
            root.func_new("echo-u8") { |n| t.u8.wrap(n) } unless skip_funcs.include?("echo-u8")
            root.func_new("echo-s16") { |n| t.s16.wrap(n) } unless skip_funcs.include?("echo-s16")
            root.func_new("echo-u16") { |n| t.u16.wrap(n) } unless skip_funcs.include?("echo-u16")
            root.func_new("echo-s64") { |n| t.s64.wrap(n) } unless skip_funcs.include?("echo-s64")
            root.func_new("echo-u64") { |n| t.u64.wrap(n) } unless skip_funcs.include?("echo-u64")
            # Float types
            root.func_new("echo-f32") { |n| t.float32.wrap(n) } unless skip_funcs.include?("echo-f32")
            root.func_new("echo-f64") { |n| t.float64.wrap(n) } unless skip_funcs.include?("echo-f64")
            # Char type
            root.func_new("echo-char") { |c| t.char.wrap(c) } unless skip_funcs.include?("echo-char")
            # Enum, variant, flags
            root.func_new("echo-enum") { |c| t.enum(["red", "green", "blue"]).wrap(c) } unless skip_funcs.include?("echo-enum")
            unless skip_funcs.include?("echo-variant")
              variant_type = t.variant(
                "circle" => t.float32,
                "rectangle" => t.tuple([t.float32, t.float32]),
                "point" => nil
              )
              root.func_new("echo-variant") { |s| variant_type.wrap(s) }
            end
            root.func_new("echo-flags") { |p| t.flags(["read", "write", "execute"]).wrap(p) } unless skip_funcs.include?("echo-flags")
          end

          # Stub math instance unless skipped
          unless skip_funcs.include?("math")
            linker.instance("math") do |math|
              math.func_new("multiply") { |a, b| t.u32.wrap(a * b) }
            end
          end
        end

        context "with primitive types" do
          it "provides a string function" do
            stub_component_imports(linker, except: :greet)

            linker.root do |root|
              root.func_new("greet") do |name|
                t.string.wrap("Hello, #{name}!")
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-greet").call("World")

            expect(result).to eq("Hello, World!")
          end

          it "provides a function with multiple params" do
            stub_component_imports(linker, except: :add)

            linker.root do |root|
              root.func_new("add") do |a, b|
                t.u32.wrap(a + b)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-add").call(42, 8)

            expect(result).to eq(50)
          end

          it "provides a function with no params" do
            stub_component_imports(linker, except: :"get-constant")

            linker.root do |root|
              root.func_new("get-constant") do
                t.u32.wrap(1234)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-constant").call

            expect(result).to eq(1234)
          end
        end

        context "with complex types" do
          it "provides a function returning a record" do
            stub_component_imports(linker, except: :"make-point")

            point_type = t.record("x" => t.s32, "y" => t.s32)

            linker.root do |root|
              root.func_new("make-point") do |x, y|
                point_type.wrap({"x" => x, "y" => y})
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-point").call(10, 20)

            expect(result).to eq({"x" => 10, "y" => 20})
          end

          it "validates field types in records" do
            stub_component_imports(linker, except: :"make-point")

            point_type = t.record("x" => t.s32, "y" => t.s32)

            linker.root do |root|
              root.func_new("make-point") do |_x, y|
                # Try to use wrong type for x field (string instead of s32)
                point_type.wrap({"x" => "not a number", "y" => y})
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-point")

            expect { func.call(10, 20) }.to raise_error(Wasmtime::Error, /expected s32, got/)
          end

          it "provides a function accepting a list" do
            stub_component_imports(linker, except: :"sum-list")

            linker.root do |root|
              root.func_new("sum-list") do |numbers|
                t.s32.wrap(numbers.sum)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-sum").call([1, 2, 3, 4, 5])

            expect(result).to eq(15)
          end

          it "provides a function with option type" do
            stub_component_imports(linker, except: :"maybe-double")

            linker.root do |root|
              root.func_new("maybe-double") do |n|
                t.option(t.u32).wrap(n.nil? ? nil : n * 2)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)

            expect(instance.get_func("test-maybe").call(21)).to eq(42)
            expect(instance.get_func("test-maybe").call(nil)).to be_nil
          end

          it "provides a function with result type" do
            stub_component_imports(linker, except: :"safe-divide")

            linker.root do |root|
              root.func_new("safe-divide") do |a, b|
                result_val = if b == 0
                  Result.error("division by zero")
                else
                  Result.ok(a / b)
                end
                t.result(t.u32, t.string).wrap(result_val)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-divide")

            expect(func.call(10, 2)).to eq(Result.ok(5))
            expect(func.call(10, 0)).to eq(Result.error("division by zero"))
          end

          it "provides a function returning a list" do
            stub_component_imports(linker, except: :"get-numbers")

            linker.root do |root|
              root.func_new("get-numbers") do
                t.list(t.s32).wrap([1, 2, 3, 4, 5])
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-get-numbers").call

            expect(result).to eq([1, 2, 3, 4, 5])
          end

          it "provides a function returning a tuple" do
            stub_component_imports(linker, except: :"make-tuple")

            tuple_type = t.tuple([t.u32, t.string, t.bool])

            linker.root do |root|
              root.func_new("make-tuple") do |n, s, b|
                tuple_type.wrap([n, s, b])
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-make-tuple").call(42, "hello", true)

            expect(result).to eq([42, "hello", true])
          end

          it "provides a function returning a tuple containing a list" do
            stub_component_imports(linker, except: :"analyze-numbers")

            tuple_type = t.tuple([t.s32, t.list(t.s32)])

            linker.root do |root|
              root.func_new("analyze-numbers") do |numbers|
                tuple_type.wrap([numbers.sum, numbers.sort])
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-analyze-numbers").call([5, 1, 3, 2, 4])

            expect(result).to eq([15, [1, 2, 3, 4, 5]])
          end

          it "provides a function with s8" do
            stub_component_imports(linker, except: :"echo-s8")

            linker.root do |root|
              root.func_new("echo-s8") { |n| t.s8.wrap(n) }
            end

            instance = linker.instantiate(store, @host_imports_component)
            expect(instance.get_func("test-s8").call(-42)).to eq(-42)
            expect(instance.get_func("test-s8").call(127)).to eq(127)
          end

          it "provides a function with u8" do
            stub_component_imports(linker, except: :"echo-u8")

            linker.root do |root|
              root.func_new("echo-u8") { |n| t.u8.wrap(n) }
            end

            instance = linker.instantiate(store, @host_imports_component)
            expect(instance.get_func("test-u8").call(255)).to eq(255)
          end

          it "provides a function with s16" do
            stub_component_imports(linker, except: :"echo-s16")

            linker.root do |root|
              root.func_new("echo-s16") { |n| t.s16.wrap(n) }
            end

            instance = linker.instantiate(store, @host_imports_component)
            expect(instance.get_func("test-s16").call(-1000)).to eq(-1000)
            expect(instance.get_func("test-s16").call(32767)).to eq(32767)
          end

          it "provides a function with u16" do
            stub_component_imports(linker, except: :"echo-u16")

            linker.root do |root|
              root.func_new("echo-u16") { |n| t.u16.wrap(n) }
            end

            instance = linker.instantiate(store, @host_imports_component)
            expect(instance.get_func("test-u16").call(65535)).to eq(65535)
          end

          it "provides a function with s64" do
            stub_component_imports(linker, except: :"echo-s64")

            linker.root do |root|
              root.func_new("echo-s64") { |n| t.s64.wrap(n) }
            end

            instance = linker.instantiate(store, @host_imports_component)
            expect(instance.get_func("test-s64").call(-9_223_372_036_854_775_808)).to eq(-9_223_372_036_854_775_808)
            expect(instance.get_func("test-s64").call(9_223_372_036_854_775_807)).to eq(9_223_372_036_854_775_807)
          end

          it "provides a function with u64" do
            stub_component_imports(linker, except: :"echo-u64")

            linker.root do |root|
              root.func_new("echo-u64") { |n| t.u64.wrap(n) }
            end

            instance = linker.instantiate(store, @host_imports_component)
            expect(instance.get_func("test-u64").call(18_446_744_073_709_551_615)).to eq(18_446_744_073_709_551_615)
          end

          it "provides a function with f32" do
            stub_component_imports(linker, except: :"echo-f32")

            linker.root do |root|
              root.func_new("echo-f32") { |n| t.float32.wrap(n) }
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-f32").call(3.14)
            expect(result).to be_within(0.01).of(3.14)
          end

          it "provides a function with f64" do
            stub_component_imports(linker, except: :"echo-f64")

            linker.root do |root|
              root.func_new("echo-f64") { |n| t.float64.wrap(n) }
            end

            instance = linker.instantiate(store, @host_imports_component)
            expect(instance.get_func("test-f64").call(3.141592653589793)).to eq(3.141592653589793)
          end

          it "provides a function with char" do
            stub_component_imports(linker, except: :"echo-char")

            linker.root do |root|
              root.func_new("echo-char") { |c| t.char.wrap(c) }
            end

            instance = linker.instantiate(store, @host_imports_component)
            expect(instance.get_func("test-char").call("A")).to eq("A")
            expect(instance.get_func("test-char").call("🎉")).to eq("🎉")
          end

          it "provides a function with enum" do
            stub_component_imports(linker, except: :"echo-enum")

            linker.root do |root|
              root.func_new("echo-enum") do |color|
                t.enum(["red", "green", "blue"]).wrap(color)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            expect(instance.get_func("test-enum").call("red")).to eq("red")
            expect(instance.get_func("test-enum").call("green")).to eq("green")
            expect(instance.get_func("test-enum").call("blue")).to eq("blue")
          end

          it "provides a function with variant" do
            stub_component_imports(linker, except: :"echo-variant")

            variant_type = t.variant(
              "circle" => t.float32,
              "rectangle" => t.tuple([t.float32, t.float32]),
              "point" => nil
            )

            linker.root do |root|
              root.func_new("echo-variant") do |shape|
                variant_type.wrap(shape)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-variant")

            result = func.call(Variant.new("circle", 5.0))
            expect(result.name).to eq("circle")
            expect(result.value).to be_within(0.01).of(5.0)

            result = func.call(Variant.new("rectangle", [10.0, 20.0]))
            expect(result.name).to eq("rectangle")
            expect(result.value).to eq([10.0, 20.0])

            result = func.call(Variant.new("point", nil))
            expect(result.name).to eq("point")
            expect(result.value).to be_nil
          end

          it "provides a function with flags" do
            stub_component_imports(linker, except: :"echo-flags")

            linker.root do |root|
              root.func_new("echo-flags") do |perms|
                t.flags(["read", "write", "execute"]).wrap(perms)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-flags")

            expect(func.call([])).to eq([])
            expect(func.call(["read"])).to eq(["read"])
            expect(func.call(["read", "write"])).to eq(["read", "write"])
            expect(func.call(["read", "write", "execute"])).to eq(["read", "write", "execute"])
          end
        end

        context "with nested instances" do
          it "provides functions in nested instances" do
            stub_component_imports(linker, except: :math)

            linker.instance("math") do |math|
              math.func_new("multiply") do |a, b|
                t.u32.wrap(a * b)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-multiply").call(6, 7)

            expect(result).to eq(42)
          end
        end

        context "with stateful closures" do
          it "maintains state across calls" do
            counter = 0

            stub_component_imports(linker, except: :"get-constant")

            linker.root do |root|
              root.func_new("get-constant") do
                counter += 1
                t.u32.wrap(counter)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-constant")

            expect(func.call).to eq(1)
            expect(func.call).to eq(2)
            expect(func.call).to eq(3)
          end

          it "allows accessing Ruby objects" do
            log = []

            stub_component_imports(linker, except: :greet)

            linker.root do |root|
              root.func_new("greet") do |name|
                log << name
                t.string.wrap("Hello, #{name}!")
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-greet")

            func.call("Alice")
            func.call("Bob")

            expect(log).to eq(["Alice", "Bob"])
          end
        end

        context "error handling" do
          it "propagates Ruby exceptions" do
            stub_component_imports(linker, except: :"get-constant")

            linker.root do |root|
              root.func_new("get-constant") do
                raise "Something went wrong"
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-constant")

            expect { func.call }.to raise_error(RuntimeError, /Something went wrong/)
          end

          it "validates return values match declared types" do
            stub_component_imports(linker, except: :add)

            linker.root do |root|
              root.func_new("add") do |_a, _b|
                t.u32.wrap("not a number")
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-add")

            expect { func.call(1, 2) }.to raise_error(Wasmtime::Error, /expected u32, got/)
          end

          it "raises clear error when return value is not wrapped" do
            stub_component_imports(linker, except: :add)

            linker.root do |root|
              root.func_new("add") do |a, b|
                a + b  # Forgot to wrap with Type::U32.wrap()
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-add")

            expect { func.call(1, 2) }.to raise_error(
              TypeError,
              /host function must return wrapped value/
            )
          end

          it "raises Wasmtime error when wrapper type mismatches component expectation" do
            stub_component_imports(linker, except: :add)

            linker.root do |root|
              root.func_new("add") do |a, b|
                # Component expects u32, but we return s32
                t.s32.wrap(a + b)
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-add")

            # The component crashes at runtime when the wrong type is returned
            expect { func.call(1, 2) }.to raise_error(Wasmtime::Error, /error while executing at wasm/i)
          end
        end
      end
    end
  end
end
