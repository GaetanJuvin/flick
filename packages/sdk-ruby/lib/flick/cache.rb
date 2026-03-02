# frozen_string_literal: true

module Flick
  class Cache
    def initialize
      @flags = {}
      @version = nil
      @mutex = Mutex.new
    end

    # Returns true if the cache was updated, false if version matched.
    def update(flags, version)
      @mutex.synchronize do
        return false if @version == version

        @flags = {}
        flags.each { |f| @flags[f.key] = f }
        @version = version
        true
      end
    end

    def get(key)
      @mutex.synchronize { @flags[key] }
    end

    def get_all
      @mutex.synchronize { @flags.values.dup }
    end

    def version
      @mutex.synchronize { @version }
    end

    def empty?
      @mutex.synchronize { @flags.empty? }
    end
  end
end
