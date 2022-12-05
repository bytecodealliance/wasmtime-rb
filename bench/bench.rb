require "benchmark/ips"
require "wasmtime"

module Bench
  extend(self)

  def ips
    Benchmark.ips do |x|
      yield(x)

      x.config(time: 0, warmup: 0) if ENV["CI"]
    end
  end
end
