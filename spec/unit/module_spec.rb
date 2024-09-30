require "spec_helper"
require "tempfile"
require "securerandom"
require "pathname"

module Wasmtime
  RSpec.describe Module do
    it "can be serialized and deserialized" do
      mod = Module.new(engine, wat)
      serialized = mod.serialize
      deserialized = Module.deserialize(engine, serialized)
      expect(deserialized.serialize).to eq(serialized)
    end

    describe ".from_file" do
      it "loads the module" do
        mod = Module.from_file(engine, "spec/fixtures/empty.wat")
        expect(mod).to be_instance_of(Module)
      end

      it "tracks memory usage" do
        _, increase_bytes = measure_gc_stat(:malloc_increase_bytes) do
          Module.from_file(engine, "spec/fixtures/empty.wat")
        end

        # This is a rough estimate of the memory usage of the module, subject to compiler changes
        expect(increase_bytes).to be > 3000
      end
    end

    describe ".deserialize_file" do
      include_context(:tmpdir)
      let(:tmpdir) { Dir.mktmpdir }

      after(:each) do
        FileUtils.rm_rf(tmpdir)
      rescue Errno::EACCES => e
        warn "WARN: Failed to remove #{tmpdir} (#{e})"
      end

      it("can deserialize a module from a file") do
        tmpfile = create_tmpfile(Module.new(engine, "(module)").serialize)
        mod = Module.deserialize_file(engine, tmpfile)

        expect(mod.serialize).to eq(Module.new(engine, "(module)").serialize)
      end

      it "deserialize from a module multiple times" do
        tmpfile = create_tmpfile(Module.new(engine, wat).serialize)

        mod_one = Module.deserialize_file(engine, tmpfile)
        mod_two = Module.deserialize_file(engine, tmpfile)
        expected = Module.new(engine, wat).serialize

        expect(mod_one.serialize).to eq(expected)
        expect(mod_two.serialize).to eq(expected)
      end

      it "tracks memory usage" do
        tmpfile = create_tmpfile(Module.new(engine, "(module)").serialize)
        mod, increase_bytes = measure_gc_stat(:malloc_increase_bytes) { Module.deserialize_file(engine, tmpfile) }

        expect(increase_bytes).to be > File.size(tmpfile)
        expect(mod).to be_a(Wasmtime::Module)
      end

      def create_tmpfile(content)
        uuid = SecureRandom.uuid
        path = File.join(tmpdir, "deserialize-file-test-#{uuid}.so")
        File.binwrite(path, content)
        path
      end
    end

    describe ".deserialize" do
      it "raises on invalid module" do
        expect { Module.deserialize(engine, "foo") }
          .to raise_error(Wasmtime::Error)
      end

      it "tracks memory usage" do
        serialized = Module.new(engine, wat).serialize
        mod, increase_bytes = measure_gc_stat(:malloc_increase_bytes) { Module.deserialize(engine, serialized) }

        expect(increase_bytes).to be > serialized.bytesize
        expect(mod).to be_a(Wasmtime::Module)
      end
    end

    describe "#imports" do
      cases = {
        f: [:to_func_type, FuncType, {params: [:i32], results: [:i32]}],
        m: [:to_memory_type, MemoryType, {min_size: 1, max_size: nil}],
        t: [:to_table_type, TableType, {type: :funcref, min_size: 1, max_size: nil}],
        g: [:to_global_type, GlobalType, {const?: false, var?: true, type: :i32}]
      }

      cases.each do |name, (meth, klass, calls)|
        describe "##{meth}" do
          it "returns a type that is an instance of #{klass}" do
            import = get_import_by_name(name)
            expect(import["type"].public_send(meth)).to be_instance_of(klass)
          end

          it "raises an error when extern type is not a #{klass}" do
            import = get_import_by_name(name)
            invalid_methods = cases.values.map(&:first) - [meth]

            invalid_methods.each do |invalid_method|
              expect { import["type"].public_send(invalid_method) }.to raise_error(Wasmtime::ConversionError)
            end
          end

          it "has a type that responds to the expected methods for #{klass}" do
            import = get_import_by_name(name)
            extern_type = import["type"].public_send(meth)

            calls.each do |(meth_name, expected_return)|
              expect(extern_type.public_send(meth_name)).to eq(expected_return)
            end
          end
        end
      end

      it "has a module name" do
        mod_with_imports = new_import_module
        imports = mod_with_imports.imports

        imports.each do |import|
          expect(import["module"]).to eq("env")
        end
      end

      it "returns an empty array for a module with no imports" do
        mod = Module.new(engine, "(module)")

        expect(mod.imports).to be_an(Array)
        expect(mod.imports).to be_empty
      end

      def get_import_by_name(name)
        mod_with_imports = new_import_module
        imports = mod_with_imports.imports
        imports.find { _1["name"] == name.to_s }
      end

      def new_import_module
        Module.new(engine, <<~WAT
          (module
            (import "env" "f" (func (param i32) (result i32)))
            (import "env" "m" (memory 1))
            (import "env" "t" (table 1 funcref))
            (global (import "env" "g") (mut i32))
          )
        WAT
        )
      end
    end
  end
end
