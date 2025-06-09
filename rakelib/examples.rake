namespace :examples do
  task all: :compile

  Dir.glob("examples/*.rb").each do |path|
    task_name = File.basename(path, ".rb")

    desc "Run #{path}"
    task task_name do
      sh "ruby -Ilib #{path}"
      puts
    end

    task all: task_name
  end
end

desc "Run all the examples"
task examples: "examples:all"
