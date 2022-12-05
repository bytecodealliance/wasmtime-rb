namespace :bench do
  task all: :compile

  Dir.glob("bench/*.rb").each do |path|
    task_name = File.basename(path, ".rb")

    desc "Run #{path} benchmark"
    task task_name do
      sh "ruby -Ilib #{path}"
      puts
    end

    task all: task_name
  end
end

desc "Run all benchmarks"
task bench: "bench:all"
