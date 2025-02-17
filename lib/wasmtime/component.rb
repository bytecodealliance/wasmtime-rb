# To prevent double loading of this file when `ruby-api` is enabled
return if defined?(Wasmtime::Component::Result)

module Wasmtime
  module Component
    # Represents a component model's +result<O, E>+ type.
    class Result
      class << self
        # Construct an ok result.
        # @param ok [Object] the ok value
        # @return [Result]
        def ok(ok)
          new(true, ok)
        end

        # Construct an error result.
        # @param error [Object] the error value
        # @return [Result]
        def error(error)
          new(false, error)
        end

        private :new
      end

      # Returns the ok value of this Result if it is {#ok?}, otherwise raises.
      # @raise [UncheckedResult] if this is an error
      # @return [Object]
      def ok
        raise UncheckedResult, "expected ok, was error" unless ok?

        @value
      end

      # Returns the error value of this Result if it is {#error?}, otherwise raises.
      # @raise [UncheckedResult] if this is an ok
      # @return [Object]
      def error
        raise UncheckedResult, "expected error, was ok" unless error?

        @value
      end

      # @return [Boolean] Whether the result is ok
      def ok?
        @ok
      end

      # @return [Boolean] Whether the result is an error
      def error?
        !@ok
      end

      def ==(other)
        eql?(other)
      end

      def eql?(other)
        return false unless self.class == other.class
        return false unless ok? == other.ok?

        if ok?
          ok == other.ok
        else
          error == other.error
        end
      end

      def hash
        [self.class, @ok, @value].hash
      end

      def initialize(ok, value)
        @ok = ok
        @value = value
      end

      class UncheckedResult < Wasmtime::Error; end

      # Hide the constructor from YARD's doc so that `.ok` or
      # `.error` is used over `.new`.
      private :initialize
    end

    # Represents a value for component model's variant case.
    # A variant case has a name that uniquely identify the case within the
    # variant and optionally a value.
    #
    # @example Constructing variants
    #   # Given the following variant:
    #   # variant filter {
    #   #     all,
    #   #     none,
    #   #     lt(u32),
    #   # }
    #
    #   Variant.new("all")
    #   Variant.new("none")
    #   Variant.new("lt", 100)
    class Variant
      # The name of the variant case
      # @return [String]
      attr_reader :name

      # The optional payload of the variant case
      # @return [Object]
      attr_reader :value

      # @param name [String] the name of variant case
      # @param value [Object] the optional payload of the variant case
      def initialize(name, value = nil)
        @name = name
        @value = value
      end

      def ==(other)
        eql?(other)
      end

      def eql?(other)
        self.class == other.class &&
          name == other.name &&
          value == other.value
      end

      def hash
        [self.class, @name, @value].hash
      end
    end
  end
end
