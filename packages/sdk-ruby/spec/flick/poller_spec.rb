# frozen_string_literal: true

require "spec_helper"

RSpec.describe Flick::Poller do
  let(:base_url) { "https://flick.example.com/api/v1" }
  let(:sdk_key) { "flk_test_key" }
  let(:config_response) do
    {
      data: {
        environment: "production",
        version: "v1",
        flags: [
          { key: "my-flag", gate_type: "boolean", enabled: true, gate_config: {}, groups: [] }
        ]
      }
    }
  end

  describe "#poll" do
    it "fetches config and calls on_data" do
      received_config = nil

      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(status: 200, body: config_response.to_json, headers: { "Content-Type" => "application/json" })

      poller = described_class.new(
        base_url: base_url,
        sdk_key: sdk_key,
        interval: 300, # long interval so schedule doesn't fire during test
        on_data: ->(config) { received_config = config },
        on_error: ->(_e) {}
      )

      poller.start
      poller.stop

      expect(received_config).to be_a(Flick::FullFlagConfig)
      expect(received_config.flags.first.key).to eq("my-flag")
      expect(received_config.version).to eq("v1")
    end

    it "sends Authorization header" do
      stub = stub_request(:get, "#{base_url}/evaluate/config")
        .with(headers: { "Authorization" => "Bearer #{sdk_key}" })
        .to_return(status: 200, body: config_response.to_json)

      poller = described_class.new(
        base_url: base_url, sdk_key: sdk_key, interval: 300,
        on_data: ->(_) {}, on_error: ->(_) {}
      )
      poller.start
      poller.stop

      expect(stub).to have_been_requested
    end

    it "handles 304 Not Modified" do
      data_called = false

      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(status: 304)

      poller = described_class.new(
        base_url: base_url, sdk_key: sdk_key, interval: 300,
        on_data: ->(_) { data_called = true },
        on_error: ->(_) {}
      )
      poller.start
      poller.stop

      expect(data_called).to be false
    end

    it "sends If-None-Match with etag" do
      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(
          status: 200,
          body: config_response.to_json,
          headers: { "ETag" => '"abc123"' }
        )

      second_stub = stub_request(:get, "#{base_url}/evaluate/config")
        .with(headers: { "If-None-Match" => '"abc123"' })
        .to_return(status: 304)

      poller = described_class.new(
        base_url: base_url, sdk_key: sdk_key, interval: 0.01,
        on_data: ->(_) {}, on_error: ->(_) {}
      )
      poller.start
      sleep(0.1)
      poller.stop

      expect(second_stub).to have_been_requested.at_least_once
    end

    it "calls on_error on HTTP failure" do
      received_error = nil

      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(status: 500)

      poller = described_class.new(
        base_url: base_url, sdk_key: sdk_key, interval: 300,
        on_data: ->(_) {},
        on_error: ->(e) { received_error = e }
      )
      poller.start
      poller.stop

      expect(received_error).to be_a(Flick::PollingError)
      expect(received_error.message).to include("500")
    end

    it "calls on_error on network failure" do
      received_error = nil

      stub_request(:get, "#{base_url}/evaluate/config")
        .to_raise(Errno::ECONNREFUSED)

      poller = described_class.new(
        base_url: base_url, sdk_key: sdk_key, interval: 300,
        on_data: ->(_) {},
        on_error: ->(e) { received_error = e }
      )
      poller.start
      poller.stop

      expect(received_error).to be_a(Errno::ECONNREFUSED)
    end
  end
end
