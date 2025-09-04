require "spec_helper"
require "json"

module Wasmtime
  RSpec.describe "WASI" do
    include_context(:tmpdir)

    before(:all) do
      # Debug engine opts on the engine currently fails assertion: range_start < range_end
      @engine = GLOBAL_ENGINE

      # Compile module only once for speed
      @compiled_wasi_module = @engine.precompile_module(IO.binread("spec/fixtures/wasi-debug.wasm"))
      @compiled_wasi_deterministic_module = @engine.precompile_module(IO.binread("spec/fixtures/wasi-deterministic.wasm"))
      @compiled_wasi_fs_module = @engine.precompile_module(IO.binread("spec/fixtures/wasi-fs.wasm"))

      @compiled_wasi_component = @engine.precompile_component(IO.binread("spec/fixtures/wasi-debug-p2.wasm"))
      @compiled_wasi_deterministic_component = @engine.precompile_component(IO.binread("spec/fixtures/wasi-deterministic-p2.wasm"))
      @compiled_wasi_fs_component = @engine.precompile_component(IO.binread("spec/fixtures/wasi-fs-p2.wasm"))
    end

    describe "Linker.new" do
      it "accepts a wasi kwarg to define WASI imports" do
        linker = Linker.new(@engine)
        WASI::P1.add_to_linker_sync(linker)
        item = linker.get(Store.new(@engine), "wasi_snapshot_preview1", "environ_get")
        expect(item).not_to be nil
      end
    end

    describe "Linker#instantiate" do
      it "prevents panic when Store doesn't have a Wasi config" do
        linker = Linker.new(@engine)
        WASI::P1.add_to_linker_sync(linker)
        expect { linker.instantiate(Store.new(@engine), wasi_module).invoke("_start") }
          .to raise_error(Wasmtime::Error, /Store is missing WASI p1 configuration/)
      end

      it "returns an instance that can run when store is properly configured" do
        linker = Linker.new(@engine)
        WASI::P1.add_to_linker_sync(linker)
        store = Store.new(@engine, wasi_p1_config: WasiConfig.new.set_stdin_string("some str"))
        linker.instantiate(store, wasi_module).invoke("_start")
      end
    end

    describe "Component::Linker::instantiate" do
      it "prevents panic when Store doesn't have a WASI config" do
        linker = Component::Linker.new(@engine)
        WASI::P2.add_to_linker_sync(linker)
        expect { linker.instantiate(Store.new(@engine), wasi_component) }
          .to raise_error(Wasmtime::Error, /Store is missing WASI configuration/)
      end
    end

    describe "Component::WasiCommand#new" do
      it "prevents panic when store doesn't have a WASI config" do
        linker = Component::Linker.new(@engine)
        WASI::P2.add_to_linker_sync(linker)
        expect { Component::WasiCommand.new(Store.new(@engine), wasi_component, linker) }
          .to raise_error(Wasmtime::Error, /Store is missing WASI configuration/)
      end

      it "returns an instance that can run when store is properly configured" do
        linker = Component::Linker.new(@engine)
        WASI::P2.add_to_linker_sync(linker)
        store = Store.new(@engine, wasi_config: WasiConfig.new.set_stdin_string("some str"))
        Component::WasiCommand.new(store, wasi_component, linker).call_run(store)
      end
    end

    shared_examples WasiConfig do
      it "writes std streams to files" do
        File.write(tempfile_path("stdin"), "stdin content")
        wasi_config = WasiConfig.new
          .set_stdin_file(tempfile_path("stdin"))
          .set_stdout_file(tempfile_path("stdout"))
          .set_stderr_file(tempfile_path("stderr"))

        run.call(wasi_config)

        stdout = JSON.parse(File.read(tempfile_path("stdout")))
        stderr = JSON.parse(File.read(tempfile_path("stderr")))
        expect(stdout.fetch("name")).to eq("stdout")
        expect(stderr.fetch("name")).to eq("stderr")
        expect(stdout.dig("wasi", "stdin")).to eq("stdin content")
      end

      it "writes std streams to buffers" do
        File.write(tempfile_path("stdin"), "stdin content")

        stdout_str = ""
        stderr_str = ""
        wasi_config = WasiConfig.new
          .set_stdin_file(tempfile_path("stdin"))
          .set_stdout_buffer(stdout_str, 40000)
          .set_stderr_buffer(stderr_str, 40000)

        run.call(wasi_config)

        parsed_stdout = JSON.parse(stdout_str)
        parsed_stderr = JSON.parse(stderr_str)
        expect(parsed_stdout.fetch("name")).to eq("stdout")
        expect(parsed_stderr.fetch("name")).to eq("stderr")
      end

      it "writes std streams to buffers until capacity" do
        File.write(tempfile_path("stdin"), "stdin content")

        stdout_str = ""
        stderr_str = ""
        wasi_config = WasiConfig.new
          .set_stdin_file(tempfile_path("stdin"))
          .set_stdout_buffer(stdout_str, 5)
          .set_stderr_buffer(stderr_str, 10)

        run.call(wasi_config)

        expect(stdout_str).to eq("{\"nam")
        expect(stderr_str).to eq("{\"name\":\"s")
      end

      it "frozen stdout string is not written to" do
        File.write(tempfile_path("stdin"), "stdin content")

        stdout_str = ""
        stderr_str = ""
        wasi_config = WasiConfig.new
          .set_stdin_file(tempfile_path("stdin"))
          .set_stdout_buffer(stdout_str, 40000)
          .set_stderr_buffer(stderr_str, 40000)

        stdout_str.freeze
        expect { run.call(wasi_config) }.to raise_error do |error|
          expect(error).to be_a(Wasmtime::Error)
          expect(error.message).to match(/error while executing at wasm backtrace:/)
        end

        parsed_stderr = JSON.parse(stderr_str)
        expect(stdout_str).to eq("")
        expect(parsed_stderr.fetch("name")).to eq("stderr")
      end
      it "frozen stderr string is not written to" do
        File.write(tempfile_path("stdin"), "stdin content")

        stderr_str = ""
        stdout_str = ""
        wasi_config = WasiConfig.new
          .set_stdin_file(tempfile_path("stdin"))
          .set_stderr_buffer(stderr_str, 40000)
          .set_stdout_buffer(stdout_str, 40000)

        stderr_str.freeze
        expect { run.call(wasi_config) }.to raise_error do |error|
          expect(error).to be_a(Wasmtime::Error)
          expect(error.message).to match(/error while executing at wasm backtrace:/)
        end

        expect(stderr_str).to eq("")
        expect(stdout_str).to eq("")
      end

      it "reads stdin from string" do
        env = wasi_env.call { |config| config.set_stdin_string("¡UTF-8 from Ruby!") }
        expect(env.fetch("stdin")).to eq("¡UTF-8 from Ruby!")
      end

      it "uses specified args" do
        env = wasi_env.call { |config| config.set_argv(["foo", "bar"]) }
        expect(env.fetch("args")).to eq(["foo", "bar"])
      end

      it "uses ARGV" do
        env = wasi_env.call { |config| config.set_argv(ARGV) }
        expect(env.fetch("args")).to eq(ARGV)
      end

      it "uses specified env" do
        env = wasi_env.call { |config| config.set_env("ENV_VAR" => "VAL") }
        expect(env.fetch("env").to_h).to eq("ENV_VAR" => "VAL")
      end

      it "uses ENV" do
        env = wasi_env.call { |config| config.set_env(ENV) }
        expect(env.fetch("env").to_h).to eq(ENV.to_h)
      end

      describe "#add_determinism" do
        before do
          2.times do |t|
            t += 1
            wasi_config = WasiConfig.new
              .add_determinism
              .set_stdout_file(tempfile_path("stdout-deterministic-#{t}"))
              .set_stderr_file(tempfile_path("stderr-deterministic-#{t}"))

            run_deterministic.call(wasi_config)
          end
        end

        let(:stdout1) {
          # rubocop:disable Performance/ArraySemiInfiniteRangeSlice
          output = File.read(tempfile_path("stdout-deterministic-1"))[1..].strip
          # rubocop:enable Performance/ArraySemiInfiniteRangeSlice
          JSON.parse(output)
        }
        let(:stdout2) {
          # rubocop:disable Performance/ArraySemiInfiniteRangeSlice
          output = File.read(tempfile_path("stdout-deterministic-2"))[1..].strip
          # rubocop:enable Performance/ArraySemiInfiniteRangeSlice
          JSON.parse(output)
        }
        let(:stderr1) { File.read(tempfile_path("stderr-deterministic-1")).strip }
        let(:stderr2) { File.read(tempfile_path("stderr-deterministic-2")).strip }

        it "returns the same random values" do
          expect(stdout1["rand1"]).to eq(stdout2["rand1"])
          expect(stdout1["rand2"]).to eq(stdout2["rand2"])
          expect(stdout1["rand3"]).to eq(stdout2["rand3"])
        end

        it "returns SystemTime returns UTC 0" do
          utc_start_date_str = "1970-01-01T00:00:00+00:00"
          utc_time_keys = %w[utc1 utc2]
          elapsed_time_key = "system_time1_elapsed"
          elapsed_time = "0"

          # Confirm that that time elapsed between utc1 and utc2
          expect(stdout1[elapsed_time_key]).to eq(elapsed_time)
          expect(stdout2[elapsed_time_key]).to eq(elapsed_time)

          utc_time_keys.each do |key|
            expect(stdout1[key]).to eq(utc_start_date_str)
            expect(stdout2[key]).to eq(utc_start_date_str)
          end
        end
      end

      it "writes to mapped directory" do
        Dir.mkdir(tempfile_path("tmp"))
        File.write(tempfile_path(File.join("tmp", "counter")), "0")

        wasi_config = WasiConfig.new
          .set_argv(["wasi-fs", "/tmp/counter"])
          .set_mapped_directory(tempfile_path("tmp"), "/tmp", :all, :all)

        expect { run_fs.call(wasi_config) }.not_to raise_error

        expect(File.read(tempfile_path(File.join("tmp", "counter")))).to eq("1")
      end

      it "fails to write to mapped directory if not permitted" do
        Dir.mkdir(tempfile_path("tmp"))
        File.write(tempfile_path(File.join("tmp", "counter")), "0")

        stderr_str = ""
        wasi_config = WasiConfig.new
          .set_argv(["wasi-fs", "/tmp/counter"])
          .set_stderr_buffer(stderr_str, 40000)
          .set_mapped_directory(tempfile_path("tmp"), "/tmp", :read, :read)

        expect { run_fs.call(wasi_config) }.to raise_error do |error|
          expect(error).to be_a(Wasmtime::Error)
        end

        expect(stderr_str).to match(/failed to create counter file/)

        expect(File.read(tempfile_path(File.join("tmp", "counter")))).to eq("0")
      end

      it "fails to read from mapped directory if not permitted" do
        Dir.mkdir(tempfile_path("tmp"))
        File.write(tempfile_path(File.join("tmp", "counter")), "0")

        stderr_str = ""
        wasi_config = WasiConfig.new
          .set_argv(["wasi-fs", "/tmp/counter"])
          .set_stderr_buffer(stderr_str, 40000)
          .set_mapped_directory(tempfile_path("tmp"), "/tmp", :mutate, :write)

        expect { run_fs.call(wasi_config) }.to raise_error do |error|
          expect(error).to be_a(Wasmtime::Error)
        end

        expect(stderr_str).to match(/failed to open counter file/)

        expect(File.read(tempfile_path(File.join("tmp", "counter")))).to eq("0")
      end

      it "fails to access non-mapped directories" do
        Dir.mkdir(tempfile_path("tmp"))
        File.write(tempfile_path(File.join("tmp", "counter")), "0")

        stderr_str = ""
        wasi_config = WasiConfig.new
          .set_argv(["wasi-fs", File.join(tempfile_path("tmp"), "counter")])
          .set_stderr_buffer(stderr_str, 40000)

        expect { run_fs.call(wasi_config) }.to raise_error do |error|
          expect(error).to be_a(Wasmtime::Error)
        end

        expect(stderr_str).to match(/failed to find a pre-opened file descriptor/)

        expect(File.read(tempfile_path(File.join("tmp", "counter")))).to eq("0")
      end

      it "does not accept an invalid host path" do
        wasi_config = WasiConfig.new
          .set_mapped_directory(tempfile_path("tmp"), "/tmp", :all, :all)

        expect { run_fs.call(wasi_config) }.to raise_error do |error|
          expect(error).to be_a(Wasmtime::Error)
          # error message is os-specific
        end
      end

      it "does not accept invalid permissions" do
        wasi_config = WasiConfig.new
          .set_mapped_directory(tempfile_path("tmp"), "/tmp", :mutate, :invalid_permission)

        expect { run_fs.call(wasi_config) }.to raise_error do |error|
          expect(error).to be_a(Wasmtime::Error)
          expect(error.message).to match(/Invalid file_perms: invalid_permission. Use one of :read, :write, or :all/)
        end
      end
    end

    describe "WasiConfig preview 1" do
      it_behaves_like WasiConfig do
        let(:run) { method(:run_wasi_module) }
        let(:wasi_env) { method(:wasi_module_env) }
        let(:run_deterministic) { method(:run_wasi_module_deterministic) }
        let(:run_fs) { method(:run_wasi_module_fs) }
      end
    end

    describe "WasiConfig preview 2" do
      it_behaves_like WasiConfig do
        let(:run) { method(:run_wasi_component) }
        let(:wasi_env) { method(:wasi_component_env) }
        let(:run_deterministic) { method(:run_wasi_component_deterministic) }
        let(:run_fs) { method(:run_wasi_component_fs) }
      end
    end

    def wasi_module
      Module.deserialize(@engine, @compiled_wasi_module)
    end

    def run_wasi_module(wasi_config)
      linker = Linker.new(@engine)
      WASI::P1.add_to_linker_sync(linker)
      store = Store.new(@engine, wasi_p1_config: wasi_config)
      linker.instantiate(store, wasi_module).invoke("_start")
    end

    def run_wasi_module_deterministic(wasi_config)
      linker = Linker.new(@engine)
      WASI::P1.add_to_linker_sync(linker)
      linker.use_deterministic_scheduling_functions
      store = Store.new(@engine, wasi_p1_config: wasi_config)
      linker
        .instantiate(store, Module.deserialize(@engine, @compiled_wasi_deterministic_module))
        .invoke("_start")
    end

    def run_wasi_module_fs(wasi_config)
      linker = Linker.new(@engine)
      WASI::P1.add_to_linker_sync(linker)
      store = Store.new(@engine, wasi_p1_config: wasi_config)
      linker.instantiate(store, Module.deserialize(@engine, @compiled_wasi_fs_module)).invoke("_start")
    end

    def wasi_module_env
      stdout_file = tempfile_path("stdout")

      wasi_config = WasiConfig.new
      yield(wasi_config)
      wasi_config.set_stdout_file(stdout_file)

      run_wasi_module(wasi_config)

      JSON.parse(File.read(stdout_file)).fetch("wasi")
    end

    def wasi_component
      Component::Component.deserialize(@engine, @compiled_wasi_component)
    end

    def run_wasi_component(wasi_config)
      linker = Component::Linker.new(@engine)
      WASI::P2.add_to_linker_sync(linker)
      store = Store.new(@engine, wasi_config: wasi_config)
      Component::WasiCommand.new(store, wasi_component, linker).call_run(store)
    end

    def wasi_component_env
      stdout_file = tempfile_path("stdout")

      wasi_config = WasiConfig.new
      yield(wasi_config)
      wasi_config.set_stdout_file(stdout_file)

      run_wasi_component(wasi_config)

      JSON.parse(File.read(stdout_file)).fetch("wasi")
    end

    def run_wasi_component_deterministic(wasi_config)
      linker = Component::Linker.new(@engine)
      WASI::P2.add_to_linker_sync(linker)
      store = Store.new(@engine, wasi_config: wasi_config)
      Component::WasiCommand.new(
        store,
        Component::Component.deserialize(@engine, @compiled_wasi_deterministic_component),
        linker
      ).call_run(store)
    end

    def run_wasi_component_fs(wasi_config)
      linker = Component::Linker.new(@engine)
      WASI::P2.add_to_linker_sync(linker)
      store = Store.new(@engine, wasi_config: wasi_config)
      Component::WasiCommand.new(
        store,
        Component::Component.deserialize(@engine, @compiled_wasi_fs_component),
        linker
      ).call_run(store)
    end

    def tempfile_path(name)
      File.join(tmpdir, name)
    end
  end
end
