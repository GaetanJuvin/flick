# frozen_string_literal: true

module Flick
  class Configuration
    attr_accessor :base_url, :sdk_key, :polling_interval, :default_values,
                  :on_flags_updated, :on_error

    def initialize
      @base_url = nil
      @sdk_key = nil
      @polling_interval = 30 # seconds
      @default_values = {}
      @on_flags_updated = nil
      @on_error = nil
    end

    def validate!
      raise ConfigurationError, "base_url is required" if @base_url.nil? || @base_url.empty?
      raise ConfigurationError, "sdk_key is required" if @sdk_key.nil? || @sdk_key.empty?
    end
  end
end
