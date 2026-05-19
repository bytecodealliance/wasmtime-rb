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

        context "simple host functions" do
          it "defines a function with primitives" do
            linker.root do |root|
              root.func_new("greet") do |name|
                t.string.wrap("Hello, #{name}!")
              end
            end

            expect(linker).to be_a(Linker)
          end

          it "defines a function with multiple params" do
            linker.root do |root|
              root.func_new("add") do |a, b|
                t.u32.wrap(a + b)
              end
            end

            expect(linker).to be_a(Linker)
          end

          it "defines a function with no params" do
            linker.root do |root|
              root.func_new("get-constant") do
                t.u32.wrap(42)
              end
            end

            expect(linker).to be_a(Linker)
          end

          it "defines a function with no results" do
            linker.root do |root|
              root.func_new("log") do |_msg|
                # No return value for functions with no results
              end
            end

            expect(linker).to be_a(Linker)
          end
        end

        context "nested instances" do
          it "defines functions in nested instances" do
            linker.instance("math") do |math|
              math.func_new("add") do |a, b|
                t.u32.wrap(a + b)
              end
            end

            expect(linker).to be_a(Linker)
          end
        end

        context "error cases" do
          it "requires a block" do
            expect {
              linker.root do |root|
                root.func_new("no-block")
              end
            }.to raise_error(ArgumentError, /no block given/)
          end
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
