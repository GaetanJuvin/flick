# frozen_string_literal: true

require_relative "flick/version"
require_relative "flick/errors"
require_relative "flick/models/evaluation_result"
require_relative "flick/models/flag_config"
require_relative "flick/models/full_flag_config"
require_relative "flick/murmurhash3"
require_relative "flick/operators"
require_relative "flick/evaluator"
require_relative "flick/cache"
require_relative "flick/context"
require_relative "flick/configuration"
require_relative "flick/poller"
require_relative "flick/client"

module Flick
  class << self
    # Configure the Flick SDK.
    #
    #   Flick.configure do |config|
    #     config.base_url = "https://flick-server.example.com/api/v1"
    #     config.sdk_key  = ENV["FLICK_SDK_KEY"]
    #   end
    def configure
      @configuration = Configuration.new
      yield(@configuration)
      @client = Client.new(@configuration)
    end

    # Block until the SDK has fetched its first config.
    def wait_for_ready(timeout: 10)
      ensure_configured!
      @client.wait_for_ready(timeout: timeout)
    end

    # Check if a flag is enabled.
    # Symbols auto-convert underscores to hyphens (e.g. :new_checkout → "new-checkout").
    #
    #   Flick.enabled?(:new_checkout)
    #   Flick.enabled?(:premium_feature, current_user)
    def enabled?(flag_key, actor = nil)
      ensure_configured!
      @client.enabled?(normalize_key(flag_key), actor)
    end

    # Get all flags as { "flag-key" => true/false }.
    def all_flags
      ensure_configured!
      @client.all_flags
    end

    # Shut down the SDK (stops polling).
    def close
      @client&.close
      @client = nil
      @configuration = nil
    end

    # Access the current configuration.
    def configuration
      @configuration
    end

    private

    def ensure_configured!
      raise NotConfiguredError unless @client
    end

    def normalize_key(key)
      key.to_s.tr("_", "-")
    end
  end
end
