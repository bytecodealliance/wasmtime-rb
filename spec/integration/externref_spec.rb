require "spec_helper"
require "weakref"

module Wasmtime
  RSpec.describe "Externref lifetimes" do
    let(:instance) do
      compile(<<~WAT)
        (module
          (global $saved (mut externref) (ref.null extern))
          (table $saved_table 1 externref)
          (func (export "save_global") (param externref)
            (global.set $saved (local.get 0)))
          (func (export "load_global") (result externref)
            (global.get $saved))
          (func (export "save_table") (param externref)
            (table.set $saved_table (i32.const 0) (local.get 0)))
          (func (export "load_table") (result externref)
            (table.get $saved_table (i32.const 0)))
          (func (export "clear")
            (global.set $saved (ref.null extern))
            (table.set $saved_table (i32.const 0) (ref.null extern)))
          (func (export "identity") (param externref) (result externref)
            (local.get 0))
          (func (export "pair") (param externref externref) (result externref externref)
            (local.get 0) (local.get 1))
          (func (export "trap") (param externref)
            unreachable)
          (func (export "two_args") (param externref i32))
          (func (export "is_null") (param externref) (result i32)
            (ref.is_null (local.get 0))))
      WAT
    end

    ["global", "table"].each do |kind|
      it "keeps a temporary value stored by Wasm in a #{kind} alive and relocates it" do
        weak = save_temporary(instance.export("save_#{kind}").to_func)
        store.gc
        collect_ruby
        expect(instance.invoke("load_#{kind}")).to equal(weak.__getobj__)
        expect(weak.__getobj__).to eq("temporary value")
      end

      it "releases a value removed from the guest #{kind}" do
        values = temporary_values(instance.export("save_#{kind}").to_func)
        instance.invoke("clear")
        expect(instance.invoke("load_#{kind}")).to be_nil
        collect(store)
        expect_most_collected(values, "guest #{kind}")
      end
    end

    it "marks and relocates values in Ruby-created globals and tables" do
      global = Global.var(store, :externref, +"global value")
      table = Table.new(store, :externref, +"table value", min_size: 1)
      table.grow(1, +"grown value")
      collect(store)
      expect(global.get).to eq("global value")
      expect(table.get(0)).to eq("table value")
      expect(table.get(1)).to eq("grown value")
      global.set(+"replacement global")
      table.set(0, +"replacement table")
      collect(store)
      expect(global.get).to eq("replacement global")
      expect(table.get(0)).to eq("replacement table")
    end

    it "releases overwritten globals and table entries" do
      global = Global.var(store, :externref, nil)
      table = Table.new(store, :externref, nil, min_size: 1)
      global_values = temporary_values(global.method(:set))
      table_values = temporary_values(->(value) { table.set(0, value) })
      global.set(nil)
      table.set(0, nil)
      collect(store)
      expect_most_collected(global_values, "global writes")
      expect_most_collected(table_values, "table writes")
    end

    it "does not retain values from repeated calls or reads" do
      values = temporary_values(instance.export("identity").to_func)
      collect(store)
      expect_most_collected(values, "identity calls")
    end

    it "does not keep extra roots after reading globals and tables" do
      global = Global.var(store, :externref, nil)
      table = Table.new(store, :externref, nil, min_size: 1)
      global_values = temporary_values(->(value) do
        global.set(value)
        3.times { global.get }
      end)
      table_values = temporary_values(->(value) do
        table.set(0, value)
        3.times { table.get(0) }
      end)
      global.set(nil)
      table.set(0, nil)
      collect(store)
      expect_most_collected(global_values, "global writes")
      expect_most_collected(table_values, "table writes")
    end

    it "preserves Ruby object identity and nil's non-null externref semantics" do
      [BasicObject.new, Object.new, +"string", [], {}, 42, true, false, :symbol].each do |value|
        expect(instance.invoke("identity", value)).to equal(value)
      end
      expect(instance.invoke("identity", nil)).to be_nil
      expect(instance.invoke("is_null", nil)).to eq(0)
      expect(instance.invoke("load_global")).to be_nil
    end

    it "keeps all results rooted while allocating their Ruby result array" do
      values = [+"first", +"second"]
      result = with_gc_stress { instance.invoke("pair", *values) }
      expect(result[0]).to equal(values[0])
      expect(result[1]).to equal(values[1])
    end

    it "keeps fresh host callback results retained by the guest alive" do
      weak = nil
      callback = Func.new(store, [], [:externref]) do
        value = +"callback value"
        weak = WeakRef.new(value)
        value
      end
      mod = Module.new(engine, <<~WAT)
        (module
          (import "" "create" (func $create (result externref)))
          (global $saved (mut externref) (ref.null extern))
          (func (export "save") (global.set $saved (call $create)))
          (func (export "load") (result externref) (global.get $saved)))
      WAT
      guest = Instance.new(store, mod, [callback])
      guest.invoke("save")
      collect(store)
      expect(guest.invoke("load")).to equal(weak.__getobj__)
    end

    it "supports nested calls and collection inside host callbacks" do
      identity = instance.export("identity").to_func
      nested = Func.new(store, [:externref], [:externref]) do |_caller, value|
        store.gc
        collect_ruby
        identity.call(value)
      end
      value = +"nested value"
      expect(nested.call(value)).to equal(value)
    end

    it "releases arguments after traps and partial parameter conversion failures" do
      trap = instance.export("trap").to_func
      two_args = instance.export("two_args").to_func
      trapped = temporary_values(->(value) { expect_failure(trap, value, Trap) })
      invalid = temporary_values(->(value) do
        expect_failure(->(argument) { two_args.call(argument, nil) }, value, TypeError)
      end)
      collect(store)
      expect_most_collected(trapped, "trapped arguments")
      expect_most_collected(invalid, "invalid arguments")
    end

    it "releases partially converted callback results on error" do
      values = []
      callback = Func.new(store, [], [:externref, :i32]) do
        value = +"failed callback result"
        values << WeakRef.new(value)
        [value, nil]
      end
      without_gc_stress do
        100.times { expect { callback.call }.to raise_error(Wasmtime::ResultError) }
      end
      collect(store)
      expect_most_collected(values, "failed callback results")
    end

    it "releases values from failed global writes, table writes and table growth" do
      global = Global.const(store, :externref, nil)
      table = Table.new(store, :externref, nil, min_size: 1, max_size: 1)
      writes = {
        "failed global writes" => global.method(:set),
        "failed table writes" => ->(value) { table.set(2, value) },
        "failed table growth" => ->(value) { table.grow(1, value) }
      }
      writes.each do |label, write|
        values = temporary_values(->(value) { expect_failure(write, value, Wasmtime::Error) })
        collect(store)
        expect_most_collected(values, label)
      end
    end

    it "keeps registries isolated between stores" do
      other_store = Store.new(engine)
      first = Global.var(store, :externref, +"first store")
      second = Global.var(other_store, :externref, +"second store")
      first.set(nil)
      collect(store)
      collect(other_store)
      expect(second.get).to eq("second store")
      expect(first.get).to be_nil
    end

    it "releases values when the store is collected" do
      values = without_gc_stress { Array.new(100) { temporary_store_value } }
      collect_ruby
      expect_most_collected(values, "collected store values")
    end

    it "can collect a store and values that refer back to it" do
      values = without_gc_stress { Array.new(100) { temporary_store_cycle } }
      collect_ruby
      expect_most_collected(values, "store cycles")
    end

    it "supports host callbacks while another Ruby thread collects and compacts" do
      callback = Func.new(store, [:externref], [:externref]) do |_caller, value|
        collect_ruby
        value
      end
      mod = Module.new(engine, <<~WAT)
        (module
          (import "" "callback" (func $callback (param externref) (result externref)))
          (func (export "run") (param externref) (result externref)
            (call $callback (local.get 0))))
      WAT
      run = Instance.new(store, mod, [callback]).export("run").to_func
      save = instance.export("save_global").to_func
      clear = instance.export("clear").to_func
      collector = Thread.new { 5.times { collect_ruby } }
      10.times do
        value = +"threaded value"
        expect(run.call(value)).to equal(value)
        save_temporary(save)
        clear.call
      end
      collector.value
    ensure
      collector&.join
    end

    private

    # Keep the obvious strong reference out of the example's scope. Native-stack
    # copies may still survive collection; see expect_most_collected.
    def save_temporary(callable)
      value = +"temporary value"
      weak = WeakRef.new(value)
      callable.call(value)
      weak
    end

    def temporary_values(callable)
      without_gc_stress { Array.new(100) { save_temporary(callable) } }
    end

    # RSpec's raise_error matcher retains its block. Avoid capturing each
    # temporary value in a matcher when checking failed arguments.
    def expect_failure(callable, value, error_class)
      callable.call(value)
    rescue error_class
      nil
    else
      raise "expected #{error_class}"
    end

    def temporary_store_value
      local_store = Store.new(engine)
      global = Global.var(local_store, :externref, nil)
      save_temporary(global.method(:set))
    end

    # Sometimes native-stack scanning may retain otherwise unreachable objects,
    # so verify most objects were collected.
    def expect_most_collected(values, label)
      expect(values.size).to eq(100)
      # Snapshot liveness without dereferencing the weak references, so reporting
      # a failure does not introduce strong references to their referents.
      alive_indices = values.each_index.select { |index| values[index].weakref_alive? }
      expect(alive_indices.size).to be <= 10,
        "#{label}: #{alive_indices.size}/#{values.size} weak references still alive " \
        "(maximum 10); surviving batch indices (zero-based): #{alive_indices.inspect}"
    end

    def temporary_store_cycle
      local_store = Store.new(engine)
      Global.var(local_store, :externref, [local_store])
      WeakRef.new(local_store)
    end

    def collect_ruby
      without_gc_stress do
        GC.start(full_mark: true, immediate_sweep: true)
        if GC.respond_to?(:verify_compaction_references)
          GC.verify_compaction_references(toward: :empty)
        end
        GC.start(full_mark: true, immediate_sweep: true)
      end
    end

    def collect(target_store)
      target_store.gc
      collect_ruby
    end
  end
end
