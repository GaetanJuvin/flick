# frozen_string_literal: true

require "net/http"
require "uri"
require "json"

module Flick
  class Poller
    MAX_BACKOFF_SECONDS = 60

    def initialize(base_url:, sdk_key:, interval:, on_data:, on_error:)
      @base_url = base_url.chomp("/")
      @sdk_key = sdk_key
      @interval = interval
      @on_data = on_data
      @on_error = on_error
      @etag = nil
      @failure_count = 0
      @stopped = false
      @thread = nil
    end

    def start
      poll
      schedule
    end

    def stop
      @stopped = true
      @thread&.kill
      @thread = nil
    end

    private

    def schedule
      return if @stopped

      delay = if @failure_count > 0
        [@interval * (2**@failure_count), MAX_BACKOFF_SECONDS].min
      else
        @interval
      end

      @thread = Thread.new do
        sleep(delay)
        unless @stopped
          poll
          schedule
        end
      end
    end

    def poll
      uri = URI("#{@base_url}/evaluate/config")

      http = Net::HTTP.new(uri.host, uri.port)
      http.use_ssl = (uri.scheme == "https")
      http.open_timeout = 10
      http.read_timeout = 10

      request = Net::HTTP::Get.new(uri)
      request["Authorization"] = "Bearer #{@sdk_key}"
      request["Accept"] = "application/json"
      request["If-None-Match"] = @etag if @etag

      response = http.request(request)

      case response.code.to_i
      when 304
        @failure_count = 0
      when 200..299
        @etag = response["etag"] if response["etag"]
        body = JSON.parse(response.body)
        config = FullFlagConfig.from_hash(body["data"])
        @failure_count = 0
        @on_data.call(config)
      else
        raise PollingError, "Polling failed: #{response.code}"
      end
    rescue => e
      @failure_count += 1
      @on_error.call(e)
    end
  end
end
