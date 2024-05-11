require "spec_helper"

module Wasmtime
  RSpec.describe PoolingAllocationConfig do
    let(:config) { PoolingAllocationConfig.new }

    it "allows total_memories configuration" do
      config.total_memories = 1
      expect(config.inspect).to include("total_memories: 1,")
    end

    it "allows total_tables configuration" do
      config.total_tables = 1
      expect(config.inspect).to include("total_tables: 1,")
    end

    it "allows max_memories_per_module configuration" do
      config.max_memories_per_module = 1
      expect(config.inspect).to include("max_memories_per_module: 1,")
    end

    it "allows max_tables_per_module configuration" do
      config.max_tables_per_module = 1
      expect(config.inspect).to include("max_tables_per_module: 1,")
    end

    it "allows async_stack_keep_resident configuration" do
      config.async_stack_keep_resident = 1
      expect(config.inspect).to match(/async_stack_keep_resident: (16384|4096),/) # page size rounded up to 16KiB
    end

    it "allows async_stack_zeroing configuration" do
      config.async_stack_zeroing = false
      expect(config.inspect).to include("async_stack_zeroing: false,")
    end

    it "allows linear_memory_keep_resident configuration" do
      config.linear_memory_keep_resident = 1
      expect(config.inspect).to match(/linear_memory_keep_resident: (16384|4096),/) # page size rounded up to 16KiB
    end

    it "allows max_component_instance_size configuration" do
      config.max_component_instance_size = 1
      expect(config.inspect).to include("component_instance_size: 1,")
    end

    it "allows max_core_instance_size configuration" do
      config.max_core_instance_size = 1
      expect(config.inspect).to include("core_instance_size: 1,")
    end

    it "allows max_memories_per_component configuration" do
      config.max_memories_per_component = 1
      expect(config.inspect).to include("max_memories_per_component: 1,")
    end

    it "allows max_memory_protection_keys configuration" do
      config.max_memory_protection_keys = 1
      expect(config.inspect).to include("max_memory_protection_keys: 1")
    end

    it "allows max_tables_per_component configuration" do
      config.max_tables_per_component = 1
      expect(config.inspect).to include("max_tables_per_component: 1,")
    end

    it "allows max_unused_warm_slots configuration" do
      config.max_unused_warm_slots = 1
      expect(config.inspect).to include("max_unused_warm_slots: 1,")
    end

    it "allows memory_pages configuration" do
      config.memory_pages = 1
      expect(config.inspect).to include("memory_pages: 1")
    end

    it "allows memory_protection_keys configuration" do
      config.memory_protection_keys = :enable
      expect(config.inspect).to include("memory_protection_keys: Enable")
      config.memory_protection_keys = :disable
      expect(config.inspect).to include("memory_protection_keys: Disable")
      config.memory_protection_keys = :auto
      expect(config.inspect).to include("memory_protection_keys: Auto")
    end

    it "allows table_elements configuration" do
      config.table_elements = 1
      expect(config.inspect).to include("table_elements: 1,")
    end

    it "allows table_keep_resident configuration" do
      config.table_keep_resident = 1
      expect(config.inspect).to match(/table_keep_resident: (16384|4096),/) # page size rounded up to 16KiB
    end

    it "allows total_component_instances configuration" do
      config.total_component_instances = 1
      expect(config.inspect).to include("total_component_instances: 1,")
    end

    it "allows total_core_instances configuration" do
      config.total_core_instances = 1
      expect(config.inspect).to include("total_core_instances: 1,")
    end

    it "allows total_stacks configuration" do
      config.total_stacks = 1
      expect(config.inspect).to include("total_stacks: 1,")
    end

    it "allows checking memory_protection_keys_available?" do
      expect(PoolingAllocationConfig.memory_protection_keys_available?).to be(true).or(be(false))
    end
  end
end
