CLEAN.include(".rspec_status")

begin
  require "rspec/core/rake_task"

  RSpec::Core::RakeTask.new(:spec)
rescue LoadError
  # No RSpec installed
end
