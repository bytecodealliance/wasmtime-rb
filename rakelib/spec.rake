CLEAN.include(".rspec_status")

begin
  require "rspec/core/rake_task"

  RSpec::Core::RakeTask.new(:spec)
rescue LoadError
  # No RSpec installed
end

desc "Run the specs (release mode)"
task "spec:release" => ["compile:release", "spec"]

desc "Run the specs (dev mode)"
task "spec:dev" => ["compile:dev", "spec"]
