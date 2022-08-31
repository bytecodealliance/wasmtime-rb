# frozen_string_literal: true

RSpec.describe Wasmtime do
  it "has a version number" do
    expect(Wasmtime::VERSION).not_to be nil
  end
end
