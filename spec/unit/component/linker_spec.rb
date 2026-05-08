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
              root.func_new("greet", [t.string], [t.string]) do |name|
                "Hello, #{name}!"
              end
            end

            expect(linker).to be_a(Linker)
          end

          it "defines a function with multiple params" do
            linker.root do |root|
              root.func_new("add", [t.u32, t.u32], [t.u32]) do |a, b|
                a + b
              end
            end

            expect(linker).to be_a(Linker)
          end

          it "defines a function with no params" do
            linker.root do |root|
              root.func_new("get-constant", [], [t.u32]) do
                42
              end
            end

            expect(linker).to be_a(Linker)
          end

          it "defines a function with no results" do
            linker.root do |root|
              root.func_new("log", [t.string], []) do |_msg|
                nil
              end
            end

            expect(linker).to be_a(Linker)
          end
        end

        context "complex types" do
          it "defines a function with record types" do
            point_type = t.record("x" => t.s32, "y" => t.s32)

            linker.root do |root|
              root.func_new("make-point", [t.s32, t.s32], [point_type]) do |x, y|
                {"x" => x, "y" => y}
              end
            end

            expect(linker).to be_a(Linker)
          end

          it "defines a function with list types" do
            linker.root do |root|
              root.func_new("sum-list", [t.list(t.s32)], [t.s32]) do |numbers|
                numbers.sum
              end
            end

            expect(linker).to be_a(Linker)
          end

          it "defines a function with option types" do
            linker.root do |root|
              root.func_new("maybe-double", [t.option(t.u32)], [t.option(t.u32)]) do |n|
                n.nil? ? nil : n * 2
              end
            end

            expect(linker).to be_a(Linker)
          end

          it "defines a function with result types" do
            linker.root do |root|
              root.func_new(
                "safe-divide",
                [t.u32, t.u32],
                [t.result(t.u32, t.string)]
              ) do |a, b|
                if b == 0
                  Result.error("division by zero")
                else
                  Result.ok(a / b)
                end
              end
            end

            expect(linker).to be_a(Linker)
          end
        end

        context "nested instances" do
          it "defines functions in nested instances" do
            linker.instance("math") do |math|
              math.func_new("add", [t.u32, t.u32], [t.u32]) do |a, b|
                a + b
              end
            end

            expect(linker).to be_a(Linker)
          end
        end

        context "error cases" do
          it "requires a block" do
            expect {
              linker.root do |root|
                root.func_new("no-block", [], [t.u32])
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
        def stub_imports_except(linker, except: nil)
          skip_funcs = Array(except).map(&:to_s).to_set

          # Stub root functions
          linker.root do |root|
            root.func_new("greet", [t.string], [t.string]) { |name| name } unless skip_funcs.include?("greet")
            root.func_new("add", [t.u32, t.u32], [t.u32]) { |a, b| a + b } unless skip_funcs.include?("add")
            root.func_new("get-constant", [], [t.u32]) { 0 } unless skip_funcs.include?("get-constant")
            unless skip_funcs.include?("make-point")
              root.func_new("make-point", [t.s32, t.s32], [t.record("x" => t.s32, "y" => t.s32)]) do |x, y|
                {"x" => x, "y" => y}
              end
            end
            root.func_new("sum-list", [t.list(t.s32)], [t.s32]) { |nums| nums.sum } unless skip_funcs.include?("sum-list")
            root.func_new("maybe-double", [t.option(t.u32)], [t.option(t.u32)]) { |n| n } unless skip_funcs.include?("maybe-double")
            unless skip_funcs.include?("safe-divide")
              root.func_new("safe-divide", [t.u32, t.u32], [t.result(t.u32, t.string)]) do |a, b|
                Result.ok(a)
              end
            end
          end

          # Stub math instance unless skipped
          unless skip_funcs.include?("math")
            linker.instance("math") do |math|
              math.func_new("multiply", [t.u32, t.u32], [t.u32]) { |a, b| a * b }
            end
          end
        end

        context "with primitive types" do
          it "provides a string function" do
            stub_imports_except(linker, except: :greet)

            linker.root do |root|
              root.func_new("greet", [t.string], [t.string]) do |name|
                "Hello, #{name}!"
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-greet").call("World")

            expect(result).to eq("Hello, World!")
          end

          it "provides a function with multiple params" do
            stub_imports_except(linker, except: :add)

            linker.root do |root|
              root.func_new("add", [t.u32, t.u32], [t.u32]) do |a, b|
                a + b
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-add").call(42, 8)

            expect(result).to eq(50)
          end

          it "provides a function with no params" do
            stub_imports_except(linker, except: :"get-constant")

            linker.root do |root|
              root.func_new("get-constant", [], [t.u32]) do
                1234
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-constant").call

            expect(result).to eq(1234)
          end
        end

        context "with complex types" do
          it "provides a function returning a record" do
            stub_imports_except(linker, except: :"make-point")

            point_type = t.record("x" => t.s32, "y" => t.s32)

            linker.root do |root|
              root.func_new("make-point", [t.s32, t.s32], [point_type]) do |x, y|
                {"x" => x, "y" => y}
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-point").call(10, 20)

            expect(result).to eq({"x" => 10, "y" => 20})
          end

          it "provides a function accepting a list" do
            stub_imports_except(linker, except: :"sum-list")

            linker.root do |root|
              root.func_new("sum-list", [t.list(t.s32)], [t.s32]) do |numbers|
                numbers.sum
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            result = instance.get_func("test-sum").call([1, 2, 3, 4, 5])

            expect(result).to eq(15)
          end

          it "provides a function with option type" do
            stub_imports_except(linker, except: :"maybe-double")

            linker.root do |root|
              root.func_new("maybe-double", [t.option(t.u32)], [t.option(t.u32)]) do |n|
                n.nil? ? nil : n * 2
              end
            end

            instance = linker.instantiate(store, @host_imports_component)

            expect(instance.get_func("test-maybe").call(21)).to eq(42)
            expect(instance.get_func("test-maybe").call(nil)).to be_nil
          end

          it "provides a function with result type" do
            stub_imports_except(linker, except: :"safe-divide")

            linker.root do |root|
              root.func_new(
                "safe-divide",
                [t.u32, t.u32],
                [t.result(t.u32, t.string)]
              ) do |a, b|
                if b == 0
                  Result.error("division by zero")
                else
                  Result.ok(a / b)
                end
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-divide")

            expect(func.call(10, 2)).to eq(Result.ok(5))
            expect(func.call(10, 0)).to eq(Result.error("division by zero"))
          end
        end

        context "with nested instances" do
          it "provides functions in nested instances" do
            stub_imports_except(linker, except: :math)

            linker.instance("math") do |math|
              math.func_new("multiply", [t.u32, t.u32], [t.u32]) do |a, b|
                a * b
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

            stub_imports_except(linker, except: :"get-constant")

            linker.root do |root|
              root.func_new("get-constant", [], [t.u32]) do
                counter += 1
                counter
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

            stub_imports_except(linker, except: :greet)

            linker.root do |root|
              root.func_new("greet", [t.string], [t.string]) do |name|
                log << name
                "Hello, #{name}!"
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
            stub_imports_except(linker, except: :"get-constant")

            linker.root do |root|
              root.func_new("get-constant", [], [t.u32]) do
                raise "Something went wrong"
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-constant")

            expect { func.call }.to raise_error(RuntimeError, /Something went wrong/)
          end

          it "validates return values match declared types" do
            stub_imports_except(linker, except: :add)

            linker.root do |root|
              root.func_new("add", [t.u32, t.u32], [t.u32]) do |_a, _b|
                "not a number"
              end
            end

            instance = linker.instantiate(store, @host_imports_component)
            func = instance.get_func("test-add")

            expect { func.call(1, 2) }.to raise_error(Wasmtime::Error, /expected u32, got "not a number"/)
          end

          it "validates host function signatures" do
            stub_imports_except(linker, except: :add)

            linker.root do |root|
              root.func_new("add", [t.s32, t.u32], [t.u32]) do |a, b|
                a + b
              end
            end

            expect {
              linker.instantiate(store, @host_imports_component)
            }.to raise_error(Wasmtime::Error, /host function "add" parameter 0 has incompatible type/)
          end

          it "validates nested instance function signatures" do
            stub_imports_except(linker, except: :math)

            linker.instance("math") do |math|
              math.func_new("multiply", [t.s32, t.s32], [t.u32]) do |a, b|
                a * b
              end
            end

            expect {
              linker.instantiate(store, @host_imports_component)
            }.to raise_error(Wasmtime::Error, /host function "math\/multiply" parameter 0 has incompatible type/)
          end

          it "validates result type mismatches" do
            stub_imports_except(linker, except: :add)

            linker.root do |root|
              root.func_new("add", [t.u32, t.u32], [t.s32]) do |a, b|
                a + b
              end
            end

            expect {
              linker.instantiate(store, @host_imports_component)
            }.to raise_error(Wasmtime::Error, /host function "add" result 0 has incompatible type/)
          end

          it "validates complex type mismatches" do
            stub_imports_except(linker, except: :"make-point")

            # Define point with wrong field types (x should be s32, not u32)
            wrong_point_type = t.record("x" => t.u32, "y" => t.s32)

            linker.root do |root|
              root.func_new("make-point", [t.s32, t.s32], [wrong_point_type]) do |x, y|
                {"x" => x, "y" => y}
              end
            end

            expect {
              linker.instantiate(store, @host_imports_component)
            }.to raise_error(Wasmtime::Error, /host function "make-point" result 0 has incompatible type.*record field 'x' type mismatch/)
          end
        end
      end
    end
  end
end
