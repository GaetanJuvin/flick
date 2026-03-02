# frozen_string_literal: true

require "spec_helper"

RSpec.describe Flick::Client do
  let(:base_url) { "https://flick.example.com/api/v1" }
  let(:sdk_key) { "flk_test_key" }
  let(:config_response) do
    {
      data: {
        environment: "production",
        version: "v1",
        flags: [
          { key: "my-flag", gate_type: "boolean", enabled: true, gate_config: {}, groups: [] },
          { key: "disabled-flag", gate_type: "boolean", enabled: false, gate_config: {}, groups: [] },
          {
            key: "pct-flag", gate_type: "percentage", enabled: true,
            gate_config: { percentage: 100 }, groups: []
          }
        ]
      }
    }
  end

  def make_config(overrides = {})
    config = Flick::Configuration.new
    config.base_url = overrides[:base_url] || base_url
    config.sdk_key = overrides[:sdk_key] || sdk_key
    config.polling_interval = overrides[:polling_interval] || 300
    config.default_values = overrides[:default_values] || {}
    config.on_flags_updated = overrides[:on_flags_updated]
    config.on_error = overrides[:on_error]
    config
  end

  describe "#enabled?" do
    before do
      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(status: 200, body: config_response.to_json)
    end

    it "returns true for enabled boolean flags" do
      client = described_class.new(make_config)
      client.wait_for_ready(timeout: 5)
      expect(client.enabled?("my-flag")).to be true
      client.close
    end

    it "returns false for disabled flags" do
      client = described_class.new(make_config)
      client.wait_for_ready(timeout: 5)
      expect(client.enabled?("disabled-flag")).to be false
      client.close
    end

    it "returns default value for unknown flags" do
      client = described_class.new(make_config(default_values: { "unknown-flag" => true }))
      client.wait_for_ready(timeout: 5)
      expect(client.enabled?("unknown-flag")).to be true
      client.close
    end

    it "returns false for unknown flags without default" do
      client = described_class.new(make_config)
      client.wait_for_ready(timeout: 5)
      expect(client.enabled?("unknown-flag")).to be false
      client.close
    end

    it "evaluates with actor context" do
      actor = double("User", flick_id: "user-123", flick_attributes: { "plan" => "pro" })
      client = described_class.new(make_config)
      client.wait_for_ready(timeout: 5)
      expect(client.enabled?("pct-flag", actor)).to be true
      client.close
    end
  end

  describe "#wait_for_ready" do
    it "resolves after first successful poll" do
      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(status: 200, body: config_response.to_json)

      client = described_class.new(make_config)
      result = client.wait_for_ready(timeout: 5)
      expect(result).to be true
      client.close
    end

    it "resolves even on poll failure with empty cache" do
      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(status: 500)

      client = described_class.new(make_config)
      result = client.wait_for_ready(timeout: 5)
      # Ready resolves so callers don't hang, but they get defaults
      expect(result).to be true
      client.close
    end
  end

  describe "#all_flags" do
    it "returns merged defaults and cached flags" do
      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(status: 200, body: config_response.to_json)

      client = described_class.new(make_config(default_values: { "extra-flag" => true }))
      client.wait_for_ready(timeout: 5)

      flags = client.all_flags
      expect(flags["my-flag"]).to be true
      expect(flags["disabled-flag"]).to be false
      expect(flags["extra-flag"]).to be true
      client.close
    end
  end

  describe "callbacks" do
    it "calls on_flags_updated when flags change" do
      updated = false
      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(status: 200, body: config_response.to_json)

      client = described_class.new(make_config(on_flags_updated: -> { updated = true }))
      client.wait_for_ready(timeout: 5)
      expect(updated).to be true
      client.close
    end

    it "calls on_error on poll failure" do
      received_error = nil
      stub_request(:get, "#{base_url}/evaluate/config")
        .to_return(status: 500)

      client = described_class.new(make_config(on_error: ->(e) { received_error = e }))
      client.wait_for_ready(timeout: 5)
      expect(received_error).to be_a(Flick::PollingError)
      client.close
    end
  end
end
