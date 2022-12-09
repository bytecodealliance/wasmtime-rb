# frozen_string_literal: true

require "spec_helper"

RSpec.describe "Forking support" do
  let(:wat) do
    <<~WAT
      (module
        (func (export "main") (result i32)
          i32.const 42))
    WAT
  end

  context "when an engine is shared with child process" do
    it "does something reasonable" do
      skip "Deadlocking"

      engine = Wasmtime::Engine.new

      fork_result = run_in_fork(true) do
        wasmmod = Wasmtime::Module.new(engine, wat)
        store = Wasmtime::Store.new(engine)
        instance = Wasmtime::Instance.new(store, wasmmod)
        instance.invoke("main").to_i
      end

      expect(fork_result).to eq(42)
    end
  end

  context "when a module is shared with child process" do
    it "can properly invoke a func from child and parent" do
      engine = Wasmtime::Engine.new
      wasmmod = Wasmtime::Module.new(engine, wat)
      store = Wasmtime::Store.new(engine)
      instance = Wasmtime::Instance.new(store, wasmmod)

      parent_result = instance.invoke("main").to_i

      fork_result = run_in_fork(true) do
        store = Wasmtime::Store.new(engine)
        instance = Wasmtime::Instance.new(store, wasmmod)
        instance.invoke("main").to_i
      end

      expect(fork_result).to eq(42)
      expect(parent_result).to eq(42)
    end
  end

  context "when a store is created in the parent process" do
    it "can properly invoke a func from child and parent" do
      engine = Wasmtime::Engine.new
      wasmmod = Wasmtime::Module.new(engine, wat)
      store = Wasmtime::Store.new(engine)
      instance = Wasmtime::Instance.new(store, wasmmod)

      parent_result = instance.invoke("main").to_i

      fork_result = run_in_fork(true) do
        instance = Wasmtime::Instance.new(store, wasmmod)
        instance.invoke("main").to_i
      end

      expect(fork_result).to eq(42)
      expect(parent_result).to eq(42)
    end
  end

  context "when an instance is created in the parent process" do
    it "can properly invoke an instance func from child and parent" do
      engine = Wasmtime::Engine.new
      wasmmod = Wasmtime::Module.new(engine, wat)
      store = Wasmtime::Store.new(engine)
      instance = Wasmtime::Instance.new(store, wasmmod)

      parent_result = instance.invoke("main").to_i

      fork_result = run_in_fork(true) do
        instance.invoke("main").to_i
      end

      expect(fork_result).to eq(42)
      expect(parent_result).to eq(42)
    end
  end
end
