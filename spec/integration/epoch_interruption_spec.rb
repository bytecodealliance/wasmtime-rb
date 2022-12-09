module Wasmtime
  RSpec.describe "Epoch interruption" do
    let(:engine) do
      config = Config.new
      config.epoch_interruption = true
      Engine.new(config)
    end

    let(:store_deadline_0) { Store.new(engine) }
    let(:store_deadline_1) { Store.new(engine).tap { |store| store.set_epoch_deadline(1) } }

    let(:mod) do
      Module.new(engine, <<~WAT)
        (module
          (func (export "42") (result i32)
            (i32.const 42))
          (func (export "loop_forever")
            (loop br 0)))
      WAT
    end

    let(:autostart_mod) do
      Module.new(engine, <<~WAT)
        (module
          (func nop)
          (start 0))
      WAT
    end

    it "starts with epoch deadline 0 and traps immediately" do
      instance = Instance.new(store_deadline_0, mod)

      expect { instance.invoke("42") }.to raise_error(Trap) do |trap|
        expect(trap.code).to eq(:interrupt)
      end

      expect { Instance.new(store_deadline_0, autostart_mod) }.to raise_error(Trap)
    end

    it "runs to completion when epoch deadline is non-zero" do
      instance = Instance.new(store_deadline_1, mod)
      expect(instance.invoke("42")).to eq(42)

      expect { Instance.new(store_deadline_1, autostart_mod) }.not_to raise_error
    end

    it "allows incrementing epoch manually" do
      instance = Instance.new(store_deadline_1, mod)
      # No error: engine is still on epoch 0
      instance.invoke("42")

      engine.increment_epoch
      expect { instance.invoke("42") }.to raise_error(Trap)
    end

    describe "Engine timer" do
      it "prevents infinite loop from running forever" do
        instance = Instance.new(store_deadline_1, mod)
        engine.start_epoch_interval(10)
        expect { instance.invoke("loop_forever") }.to raise_error(Trap)
      end

      it "can stop a previously started timer" do
        store = Store.new(engine)
        engine.start_epoch_interval(1)
        engine.stop_epoch_interval
        store.set_epoch_deadline(1)

        sleep 2.0 / 1000 # 2 ms

        expect { Instance.new(store, autostart_mod) }.not_to raise_error
      end

      it "can start and stop timers at will" do
        engine.stop_epoch_interval
        engine.start_epoch_interval(1)
        engine.start_epoch_interval(2)
        engine.stop_epoch_interval
        engine.stop_epoch_interval
      end
    end
  end
end
