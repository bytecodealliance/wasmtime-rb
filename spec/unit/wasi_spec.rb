require "spec_helper"
require "json"

module Wasmtime
  RSpec.describe "WASI" do
    include_context(:tmpdir)

    before(:all) do
      # Compile module only once for speed
      @compiled_wasi_module = Engine.new.precompile_module(IO.binread("spec/fixtures/wasi-debug.wasm"))
    end

    let(:store) { Store.new(engine) }

    describe "Linker#define_wasi" do
      it "defines WASI imports" do
        linker = Linker.new(engine).tap(&:define_wasi)
        item = linker.get(Store.new(engine), "wasi_snapshot_preview1", "environ_get")
        expect(item).not_to be nil
      end
    end

    describe "Linker#instantiate" do
      it "prevents panic when Store doesn't have a Wasi config" do
        linker = Linker.new(engine).tap(&:define_wasi)
        expect { linker.instantiate(Store.new(engine), wasi_module).invoke("_start") }
          .to raise_error(Wasmtime::Error, /Store is missing WASI configuration/)
      end

      it "returns an instance that can run when store is properly configured" do
        linker = Linker.new(engine).tap(&:define_wasi)
        store = Store.new(engine)
        store.configure_wasi(WasiConfig.new.set_stdin_string("some str"))
        linker.instantiate(store, wasi_module).invoke("_start")
      end
    end

    # Uses the program from spec/wasi-debug to test the WASI integration
    describe "WasiConfig" do
      it "writes std streams to files" do
        File.write(tempfile_path("stdin"), "stdin content")
        wasi_config = WasiConfig.new
          .set_stdin_file(tempfile_path("stdin"))
          .set_stdout_file(tempfile_path("stdout"))
          .set_stderr_file(tempfile_path("stderr"))

        run_wasi_module(wasi_config)

        stdout = JSON.parse(File.read(tempfile_path("stdout")))
        stderr = JSON.parse(File.read(tempfile_path("stderr")))
        expect(stdout.fetch("name")).to eq("stdout")
        expect(stderr.fetch("name")).to eq("stderr")
        expect(stdout.dig("wasi", "stdin")).to eq("stdin content")
      end

      it "reads stdin from string" do
        env = wasi_module_env { |config| config.set_stdin_string("¡UTF-8 from Ruby!") }
        expect(env.fetch("stdin")).to eq("¡UTF-8 from Ruby!")
      end

      it "uses specified args" do
        env = wasi_module_env { |config| config.set_argv(["foo", "bar"]) }
        expect(env.fetch("args")).to eq(["foo", "bar"])
      end

      it "uses ARGV" do
        env = wasi_module_env { |config| config.set_argv(ARGV) }
        expect(env.fetch("args")).to eq(ARGV)
      end

      it "uses specified env" do
        env = wasi_module_env { |config| config.set_env("ENV_VAR" => "VAL") }
        expect(env.fetch("env").to_h).to eq("ENV_VAR" => "VAL")
      end

      it "uses ENV" do
        env = wasi_module_env { |config| config.set_env(ENV) }
        expect(env.fetch("env").to_h).to eq(ENV.to_h)
      end
    end

    def wasi_module
      Module.deserialize(engine, @compiled_wasi_module)
    end

    def run_wasi_module(wasi_config)
      linker = Linker.new(engine).tap(&:define_wasi)
      store.configure_wasi(wasi_config)
      linker.instantiate(store, wasi_module).invoke("_start")
    end

    def wasi_module_env
      stdout_file = tempfile_path("stdout")

      wasi_config = WasiConfig.new
      yield(wasi_config)
      wasi_config.set_stdout_file(stdout_file)

      run_wasi_module(wasi_config)

      JSON.parse(File.read(stdout_file)).fetch("wasi")
    end

    def tempfile_path(name)
      File.join(tmpdir, name)
    end
  end
end
