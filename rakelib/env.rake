namespace :env do
  desc 'Sets up environment variables "dev" builds'
  task :dev do
    ENV["RUST_BACKTRACE"] = "1"
    ENV["WASMTIME_BACKTRACE_DETAILS"] = "1"
    ENV["RB_SYS_CARGO_PROFILE"] ||= "dev"
  end

  desc 'Sets up environment variables "release" builds'
  task :release do
    ENV["RB_SYS_CARGO_PROFILE"] = "release"
  end
end
