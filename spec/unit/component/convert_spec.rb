require "spec_helper"

module Wasmtime
  module Component
    RSpec.describe "Component type conversions" do
      before(:all) do
        @types_component = Component.from_file(GLOBAL_ENGINE, "spec/fixtures/component_types.wasm")
      end

      let(:linker) { Linker.new(GLOBAL_ENGINE) }
      let(:instance) { linker.instantiate(Store.new(GLOBAL_ENGINE), @types_component) }

      describe "successful round-trips" do
        [
          ["bool", true, false],
          ["u8", 0, 2**8 - 1],
          ["s8", 0, -2**7 + 1, 2**7 - 1],
          ["u16", 0, 2**16 - 1],
          ["s16", 0, -2**15 + 1, 2**15 - 1],
          ["u32", 0, 2**32 - 1],
          ["s32", 0, -2**31 + 1, 2**31 - 1],
          ["u64", 0, 2**64 - 1],
          ["s64", 0, -2**63 + 1, 2**63 - 1],
          ["f32", 0, -5.5, 5.5],
          ["f64", 0, -5.5, 5.5],
          ["char", "0", "✅"], # char: Unicode Scalar Value
          ["string", "Olá"],
          ["list", [1, 2, 2**32 - 1]], # list<u32>
          ["record", {"x" => 1, "y" => 2}],
          ["tuple", [1, "foo"]], # tuple<u32, string>
          # TODO variant
          # TODO enum
          ["option", 0, nil], # option<u32>
          ["result", Result.ok(1), Result.error(2)], # result<u32, u32>
          ["result-unit", Result.ok(nil), Result.error(nil)]
          # TODO flags
        ].each do |type, *values|
          values.each do |v|
            it "#{type} #{v.inspect}" do
              expect(instance.invoke("id-#{type}", v)).to eq(v)
            end
          end
        end

        it "returns FLOAT::INFINITY on f32 overflow" do
          expect(instance.invoke("id-f32", 5 * 10**40)).to eq(Float::INFINITY)
        end

        it "returns FLOAT::INFINITY on f64 overflow" do
          expect(instance.invoke("id-f64", 2 * 10**310)).to eq(Float::INFINITY)
        end
      end

      # TODO resource

      describe "failures" do
        [
          ["bool", "", TypeError, /conversion of String into boolean/],
          ["bool", nil, TypeError, /conversion of NilClass into boolean/],
          ["u8", "1", TypeError, /conversion of String into Integer/],
          ["u8", -1, RangeError, /negative/],
          ["u8", 2**9, RangeError, /too big/],
          ["s8", "1", TypeError, /conversion of String into Integer/],
          ["s8", 2**8, RangeError, /too big/],
          ["u16", "1", TypeError, /conversion of String into Integer/],
          ["u16", -1, RangeError, /negative/],
          ["u16", 2**17, RangeError, /too big/],
          ["s16", "1", TypeError, /conversion of String into Integer/],
          ["s16", 2**16, RangeError, /too big/],
          ["u32", "1", TypeError, /conversion of String into Integer/],
          ["u32", -1, RangeError, /negative/],
          ["u32", 2**33, RangeError, /too big/],
          ["s32", "1", TypeError, /conversion of String into Integer/],
          ["s32", 2**32, RangeError, /too big/],
          ["u64", "1", TypeError, /conversion of String into Integer/],
          ["u64", -1, RangeError, /negative/],
          ["u64", 2**65, RangeError, /too big/],
          ["s64", "1", TypeError, /conversion of String into Integer/],
          ["s64", 2**64, RangeError, /too big/],
          ["string", 1, TypeError, /conversion of Integer into String/],
          ["string", "\xFF\xFF", EncodingError, /invalid utf-8 sequence/],
          ["char", "ab", TypeError, /too many characters in string/],
          ["record", {"x" => 1}, /struct field missing: y/],
          ["record", nil, /no implicit conversion of NilClass into Hash/],
          ["result", nil, /undefined method `ok\?/],
          ["result-unit", Result.ok(""), /expected nil for result<_, E> ok branch/],
          ["result-unit", Result.error(""), /expected nil for result<O, _> error branch/]
        ].each do |type, value, klass, msg|
          it "fails on #{type} #{value.inspect}" do
            expect { instance.invoke("id-#{type}", value) }.to raise_error(klass, msg)
          end
        end

        it "has item index in list conversion error" do
          expect { instance.invoke("id-list", [1, "foo"]) }
            .to raise_error(TypeError, /list item at index 1/)
        end

        it "has tuple index in tuple conversion error" do
          expect { instance.invoke("id-tuple", ["foo", 1]) }
            .to raise_error(TypeError, /tuple value at index 0/)
        end

        it "has field name in record conversion error" do
          expect { instance.invoke("id-record", {"y" => 1, "x" => nil}) }
            .to raise_error(TypeError, /struct field "x"/)
        end
      end
    end
  end
end
