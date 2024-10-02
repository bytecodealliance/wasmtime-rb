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
    end
  end
end
