require "yard/rake/yardoc_task"

YARD::Rake::YardocTask.new do |t|
  t.options += ["--fail-on-warn"]

  t.before = -> { require "yard" }

  t.after = -> do
    require "wasmtime"

    errors = []
    YARD::Registry.each do |yard_object|
      case yard_object.type
      when :module
        mod = Object.const_get(yard_object.path)
        errors << "Not a module: #{mod.path}" unless mod.is_a?(::Module)
      when :class
        klass = Object.const_get(yard_object.path)
        errors << "Not a class: #{klass.path}" unless klass.is_a?(::Class)
      when :method
        namespace = Object.const_get(yard_object.namespace.path)
        case yard_object.scope
        when :class
          namespace.singleton_method(yard_object.name)
        when :instance
          namespace.instance_method(yard_object.name.to_s)
        else
          # Unknown scope, we should improve this script
          errors << "unknown method scope '#{yard_object.scope}' for #{yard_object.path}"
        end
      end
    rescue NameError => e
      errors << "Documented `#{yard_object.path}` not found: \n  #{e.message.split("\n").first}"
    end

    if errors.any?
      errors.each { |error| log.warn(error) }
      exit 1
    end
  end
end

namespace :doc do
  task default: [:rustdoc, :yard]

  desc "Run YARD"
  task yard: "yard"

  desc "Generate Rust documentation as JSON"
  task :rustdoc do
    run(<<~CMD)
      cargo +nightly rustdoc \
        --target-dir tmp \
        -p ext \
        -- -Zunstable-options --output-format json \
        --document-private-items
    CMD
  end

  def run(cmd)
    system(cmd)
    fail if $? != 0
  end
end

task doc: "doc:default"
