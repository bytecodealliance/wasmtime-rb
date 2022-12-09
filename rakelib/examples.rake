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

  desc "Run rust-crate/"
  task :rust_crate do
    Dir.chdir("examples/rust-crate") do
      sh "cargo test"
    end
  end

  task all: :rust_crate
end

desc "Run all the examples"
task examples: "examples:all"
