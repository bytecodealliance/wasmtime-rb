require "spec_helper"

module Wasmtime
  module Component
    RSpec.describe Result do
      let(:ok) { Result.ok(1) }
      let(:error) { Result.error(1) }

      it "creates a new ok result" do
        expect(ok).to be_instance_of(Result)
        expect(ok).to be_ok
        expect(ok).not_to be_error
      end

      it "creates a new error result" do
        expect(error).to be_instance_of(Result)
        expect(error).to be_error
        expect(error).not_to be_ok
      end

      it "raises when accessing unchecked value" do
        expect { error.ok }.to raise_error(Result::UncheckedResult)
        expect { ok.error }.to raise_error(Result::UncheckedResult)
      end

      it "behaves like a value object" do
        expect(Result.ok(1)).to eq(Result.ok(1))
        expect(Result.ok(1).hash).to eq(Result.ok(1).hash)

        expect(Result.ok(1)).not_to eq(Result.ok(2))
        expect(Result.ok(1).hash).not_to eq(Result.ok(2).hash)
        expect(Result.ok(1).hash).not_to eq(Result.error(1).hash)
      end
    end
  end
end
