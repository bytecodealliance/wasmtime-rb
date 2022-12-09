module ForkHelper
  def run_in_fork(enable = true, &blk)
    reader, writer = IO.pipe

    pid = fork do
      reader.close
      result = blk.call
      Marshal.dump({status: :success, result: result}, writer)
    rescue => e
      puts "ERROR: #{e}"
      Marshal.dump({status: :error, result: e}, writer)
    ensure
      writer.close
    end

    writer.close

    if ENV["FORK_HELPER_TIMEOUT_KILL"] != "0"
      Process.kill("KILL", pid) unless IO.select([reader], nil, nil, 1)
    end

    result = begin
      Marshal.load(reader)
    rescue EOFError
      raise "Child process did not complete"
    end

    status = result[:status]

    if status == :error
      raise result[:result]
    else
      result[:result]
    end
  end
end
