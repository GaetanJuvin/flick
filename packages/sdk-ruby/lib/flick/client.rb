# frozen_string_literal: true

module Flick
  class Client
    def initialize(config)
      config.validate!

      @cache = Cache.new
      @default_values = config.default_values || {}
      @ready = false
      @mutex = Mutex.new
      @ready_cv = ConditionVariable.new

      @poller = Poller.new(
        base_url: config.base_url,
        sdk_key: config.sdk_key,
        interval: config.polling_interval,
        on_data: method(:handle_data).curry[config.on_flags_updated],
        on_error: method(:handle_error).curry[config.on_error]
      )

      @poller.start
    end

    # Block until first config fetch or timeout.
    # Returns true if ready with data, false if timed out or resolved with defaults.
    def wait_for_ready(timeout: 10)
      @mutex.synchronize do
        return true if @ready

        @ready_cv.wait(@mutex, timeout)
        @ready
      end
    end

    def enabled?(flag_key, actor = nil)
      flag = @cache.get(flag_key)

      unless flag
        return @default_values[flag_key] || false
      end

      context = if actor
        Context.from_actor(actor)
      else
        { key: "", attributes: {} }
      end

      result = Evaluator.evaluate(flag, context)
      result.enabled
    end

    def all_flags
      result = @default_values.dup
      @cache.get_all.each { |f| result[f.key] = f.enabled }
      result
    end

    def close
      @poller.stop
    end

    private

    def handle_data(on_flags_updated, config)
      changed = @cache.update(config.flags, config.version)

      @mutex.synchronize do
        unless @ready
          @ready = true
          @ready_cv.broadcast
        end
      end

      on_flags_updated&.call if changed
    end

    def handle_error(on_error, error)
      on_error&.call(error)

      # If first poll fails and cache is empty, resolve ready
      # so callers don't hang forever — they'll get defaults
      @mutex.synchronize do
        if !@ready && @cache.empty?
          @ready = true
          @ready_cv.broadcast
        end
      end
    end
  end
end
