# frozen_string_literal: true

module Flick
  class Error < StandardError; end

  class NotConfiguredError < Error
    def initialize
      super("Flick is not configured. Call Flick.configure first.")
    end
  end

  class ConfigurationError < Error; end

  class PollingError < Error; end
end
