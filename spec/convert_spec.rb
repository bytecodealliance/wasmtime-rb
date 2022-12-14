require "spec_helper"

module Wasmtime
  RSpec.describe "Type conversions" do
    describe "for numbers" do
      [
        [:i32, 4],
        [:i64, 2**40],
        [:f32, 5.5],
        [:f64, 5.5]
      ].each do |type, value|
        it "converts #{type} back and forth" do
          expect(roundtrip_value(type, value)).to eq(value)
        end
      end

      it "raises on i32 overflow" do
        expect { roundtrip_value(:i32, 2**50) }.to raise_error(RangeError)
      end

      it "raises on i64 overflow" do
        expect { roundtrip_value(:i64, 2**65) }.to raise_error(RangeError)
      end

      it "returns FLOAT::INFINITY on f32 overflow" do
        expect(roundtrip_value(:f32, 5 * 10**40)).to eq(Float::INFINITY)
      end

      it "returns FLOAT::INFINITY on f64 overflow" do
        expect(roundtrip_value(:f64, 2 * 10**310)).to eq(Float::INFINITY)
      end
    end

    describe "for externref" do
      let(:basic_object) { BasicObject.new }

      it("converts nil back and forth") { expect(roundtrip_value(:externref, nil)).to be_nil }
      it("converts string back and forth") { expect(roundtrip_value(:externref, "foo")).to eq("foo") }
      it "converts BasicObject back and forth" do
        expect(roundtrip_value(:externref, basic_object)).to equal(basic_object)
      end
    end

    it "converts ref.null to nil" do
      instance = compile(<<~WAT)
        (module
          (func (export "main") (result externref)
            ref.null extern))
      WAT
      expect(instance.invoke("main")).to be_nil
    end

    describe "for funcref" do
      it "converts back and forth" do
        store = Store.new(engine)
        f1 = Func.new(store, [], []) {}
        f2 = Func.new(store, [:funcref], [:funcref]) { |_, arg1| arg1 }
        returned_func = f2.call(f1)
        expect(returned_func).to be_instance_of(Func)
      end

      it "converts ref.null to nil" do
        instance = compile(<<~WAT)
          (module
            (func (export "main") (result funcref)
              ref.null func))
        WAT
        expect(instance.invoke("main")).to be_nil
      end
    end

    private

    def roundtrip_value(type, value)
      Func
        .new(Store.new(engine), [type], [type]) do |_caller, arg|
          arg
        end
        .call(value)
    end
  end
end
