# frozen_string_literal: true

require "spec_helper"

RSpec.describe Flick::Evaluator do
  describe ".evaluate" do
    let(:context) { { key: "user-123", attributes: {} } }

    context "when flag is nil" do
      it "returns flag_not_found" do
        result = described_class.evaluate(nil, context)
        expect(result.enabled).to be false
        expect(result.reason).to eq("flag_not_found")
      end
    end

    context "when flag is disabled" do
      it "returns flag_disabled" do
        flag = Flick::FlagConfig.new(
          key: "my-flag", gate_type: "boolean", enabled: false,
          gate_config: {}, groups: []
        )
        result = described_class.evaluate(flag, context)
        expect(result.enabled).to be false
        expect(result.reason).to eq("flag_disabled")
        expect(result.flag_key).to eq("my-flag")
      end
    end

    context "boolean gate" do
      it "returns true when enabled" do
        flag = Flick::FlagConfig.new(
          key: "my-flag", gate_type: "boolean", enabled: true,
          gate_config: {}, groups: []
        )
        result = described_class.evaluate(flag, context)
        expect(result.enabled).to be true
        expect(result.reason).to eq("boolean_on")
      end
    end

    context "percentage gate" do
      it "enables when hash < percentage" do
        flag = Flick::FlagConfig.new(
          key: "my-flag", gate_type: "percentage", enabled: true,
          gate_config: { "percentage" => 100 }, groups: []
        )
        result = described_class.evaluate(flag, context)
        expect(result.enabled).to be true
        expect(result.reason).to eq("percentage_match")
      end

      it "disables when hash >= percentage" do
        flag = Flick::FlagConfig.new(
          key: "my-flag", gate_type: "percentage", enabled: true,
          gate_config: { "percentage" => 0 }, groups: []
        )
        result = described_class.evaluate(flag, context)
        expect(result.enabled).to be false
        expect(result.reason).to eq("percentage_miss")
      end

      it "produces deterministic results for same key+context" do
        flag = Flick::FlagConfig.new(
          key: "my-flag", gate_type: "percentage", enabled: true,
          gate_config: { "percentage" => 50 }, groups: []
        )
        results = 10.times.map { described_class.evaluate(flag, context).enabled }
        expect(results.uniq.size).to eq(1) # All same
      end
    end

    context "group gate" do
      it "returns group_miss when no groups" do
        flag = Flick::FlagConfig.new(
          key: "my-flag", gate_type: "group", enabled: true,
          gate_config: {}, groups: []
        )
        result = described_class.evaluate(flag, context)
        expect(result.enabled).to be false
        expect(result.reason).to eq("group_miss")
      end

      it "matches when any group matches (OR)" do
        flag = Flick::FlagConfig.new(
          key: "my-flag", gate_type: "group", enabled: true,
          gate_config: {},
          groups: [
            Flick::FlagGroup.new(
              id: "g1",
              rules: [Flick::GroupRule.new(attribute: "plan", operator: "eq", value: "enterprise")]
            ),
            Flick::FlagGroup.new(
              id: "g2",
              rules: [Flick::GroupRule.new(attribute: "plan", operator: "eq", value: "pro")]
            )
          ]
        )
        ctx = { key: "user-123", attributes: { "plan" => "pro" } }
        result = described_class.evaluate(flag, ctx)
        expect(result.enabled).to be true
        expect(result.reason).to eq("group_match")
      end

      it "requires all rules within a group to match (AND)" do
        flag = Flick::FlagConfig.new(
          key: "my-flag", gate_type: "group", enabled: true,
          gate_config: {},
          groups: [
            Flick::FlagGroup.new(
              id: "g1",
              rules: [
                Flick::GroupRule.new(attribute: "plan", operator: "eq", value: "pro"),
                Flick::GroupRule.new(attribute: "country", operator: "eq", value: "US")
              ]
            )
          ]
        )
        ctx = { key: "user-123", attributes: { "plan" => "pro", "country" => "CA" } }
        result = described_class.evaluate(flag, ctx)
        expect(result.enabled).to be false
        expect(result.reason).to eq("group_miss")
      end
    end

    context "unknown gate type" do
      it "returns flag_not_found" do
        flag = Flick::FlagConfig.new(
          key: "my-flag", gate_type: "unknown", enabled: true,
          gate_config: {}, groups: []
        )
        result = described_class.evaluate(flag, context)
        expect(result.enabled).to be false
        expect(result.reason).to eq("flag_not_found")
      end
    end
  end
end
