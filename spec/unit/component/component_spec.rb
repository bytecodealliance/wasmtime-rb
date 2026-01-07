require "spec_helper"

module Wasmtime
  module Component
    RSpec.describe Component do
      it "can be serialized and deserialized" do
        component = Component.new(engine, "(component)")
        serialized = component.serialize
        deserialized = Component.deserialize(engine, serialized)
        expect(deserialized.serialize).to eq(serialized)
      end

      describe ".from_file" do
        it "loads the Component" do
          component = Component.from_file(engine, "spec/fixtures/empty_component.wat")
          expect(component).to be_instance_of(Component)
        end

        it "tracks memory usage" do
          _, increase_bytes = measure_gc_stat(:malloc_increase_bytes) do
            Component.from_file(engine, "spec/fixtures/empty_component.wat")
          end

          # This is a rough estimate of the memory usage of the Component, subject to compiler changes
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

        it("can deserialize a Component from a file") do
          tmpfile = create_tmpfile(Component.new(engine, "(component)").serialize)
          component = Component.deserialize_file(engine, tmpfile)

          expect(component.serialize).to eq(Component.new(engine, "(component)").serialize)
        end

        it "deserialize from a Component multiple times" do
          tmpfile = create_tmpfile(Component.new(engine, "(component)").serialize)

          component_one = Component.deserialize_file(engine, tmpfile)
          component_two = Component.deserialize_file(engine, tmpfile)
          expected = Component.new(engine, "(component)").serialize

          expect(component_one.serialize).to eq(expected)
          expect(component_two.serialize).to eq(expected)
        end

        it "tracks memory usage" do
          tmpfile = create_tmpfile(Component.new(engine, "(component)").serialize)
          component, increase_bytes = measure_gc_stat(:malloc_increase_bytes) { Component.deserialize_file(engine, tmpfile) }

          expect(increase_bytes).to be > 0
          expect(component).to be_a(Component)
        end

        def create_tmpfile(content)
          uuid = SecureRandom.uuid
          path = File.join(tmpdir, "deserialize-file-test-#{uuid}.so")
          File.binwrite(path, content)
          path
        end
      end

      describe ".deserialize" do
        it "raises on invalid Component" do
          expect { Component.deserialize(engine, "foo") }
            .to raise_error(Wasmtime::Error)
        end

        it "tracks memory usage" do
          serialized = Component.new(engine, "(component)").serialize
          component, increase_bytes = measure_gc_stat(:malloc_increase_bytes) { Component.deserialize(engine, serialized) }

          expect(increase_bytes).to be > 0
          expect(component).to be_a(Wasmtime::Component::Component)
        end
      end
    end
  end
end
