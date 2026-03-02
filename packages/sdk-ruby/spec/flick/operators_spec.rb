# frozen_string_literal: true

require "spec_helper"

RSpec.describe Flick::Operators do
  def rule(attribute:, operator:, value:)
    Flick::GroupRule.new(attribute: attribute, operator: operator, value: value)
  end

  describe ".match_rule" do
    context "eq operator" do
      it "matches equal string values" do
        expect(described_class.match_rule(rule(attribute: "plan", operator: "eq", value: "pro"), { "plan" => "pro" })).to be true
      end

      it "does not match different values" do
        expect(described_class.match_rule(rule(attribute: "plan", operator: "eq", value: "pro"), { "plan" => "free" })).to be false
      end

      it "coerces to string for comparison" do
        expect(described_class.match_rule(rule(attribute: "count", operator: "eq", value: "5"), { "count" => 5 })).to be true
      end
    end

    context "neq operator" do
      it "matches different values" do
        expect(described_class.match_rule(rule(attribute: "plan", operator: "neq", value: "pro"), { "plan" => "free" })).to be true
      end

      it "does not match equal values" do
        expect(described_class.match_rule(rule(attribute: "plan", operator: "neq", value: "pro"), { "plan" => "pro" })).to be false
      end
    end

    context "in operator" do
      it "matches when value is in list" do
        expect(described_class.match_rule(rule(attribute: "country", operator: "in", value: ["US", "CA"]), { "country" => "US" })).to be true
      end

      it "does not match when value is not in list" do
        expect(described_class.match_rule(rule(attribute: "country", operator: "in", value: ["US", "CA"]), { "country" => "GB" })).to be false
      end

      it "returns false when value is not an array" do
        expect(described_class.match_rule(rule(attribute: "country", operator: "in", value: "US"), { "country" => "US" })).to be false
      end
    end

    context "not_in operator" do
      it "matches when value is not in list" do
        expect(described_class.match_rule(rule(attribute: "country", operator: "not_in", value: ["US", "CA"]), { "country" => "GB" })).to be true
      end

      it "does not match when value is in list" do
        expect(described_class.match_rule(rule(attribute: "country", operator: "not_in", value: ["US", "CA"]), { "country" => "US" })).to be false
      end

      it "returns true when value is not an array" do
        expect(described_class.match_rule(rule(attribute: "country", operator: "not_in", value: "US"), { "country" => "US" })).to be true
      end
    end

    context "contains operator" do
      it "matches substring" do
        expect(described_class.match_rule(rule(attribute: "email", operator: "contains", value: "@example"), { "email" => "user@example.com" })).to be true
      end

      it "does not match missing substring" do
        expect(described_class.match_rule(rule(attribute: "email", operator: "contains", value: "@other"), { "email" => "user@example.com" })).to be false
      end
    end

    context "starts_with operator" do
      it "matches prefix" do
        expect(described_class.match_rule(rule(attribute: "name", operator: "starts_with", value: "John"), { "name" => "John Doe" })).to be true
      end
    end

    context "ends_with operator" do
      it "matches suffix" do
        expect(described_class.match_rule(rule(attribute: "email", operator: "ends_with", value: ".com"), { "email" => "user@example.com" })).to be true
      end
    end

    context "gt operator" do
      it "matches greater value" do
        expect(described_class.match_rule(rule(attribute: "age", operator: "gt", value: 18), { "age" => 25 })).to be true
      end

      it "does not match equal value" do
        expect(described_class.match_rule(rule(attribute: "age", operator: "gt", value: 25), { "age" => 25 })).to be false
      end
    end

    context "gte operator" do
      it "matches equal value" do
        expect(described_class.match_rule(rule(attribute: "age", operator: "gte", value: 25), { "age" => 25 })).to be true
      end
    end

    context "lt operator" do
      it "matches lesser value" do
        expect(described_class.match_rule(rule(attribute: "age", operator: "lt", value: 30), { "age" => 25 })).to be true
      end
    end

    context "lte operator" do
      it "matches equal value" do
        expect(described_class.match_rule(rule(attribute: "age", operator: "lte", value: 25), { "age" => 25 })).to be true
      end
    end

    context "regex operator" do
      it "matches pattern" do
        expect(described_class.match_rule(rule(attribute: "email", operator: "regex", value: ".*@example\\.com$"), { "email" => "user@example.com" })).to be true
      end

      it "returns false for invalid regex" do
        expect(described_class.match_rule(rule(attribute: "email", operator: "regex", value: "[invalid"), { "email" => "user@example.com" })).to be false
      end
    end

    context "missing attribute" do
      it "returns false when attribute is missing" do
        expect(described_class.match_rule(rule(attribute: "missing", operator: "eq", value: "x"), {})).to be false
      end
    end

    context "unknown operator" do
      it "returns false" do
        expect(described_class.match_rule(rule(attribute: "x", operator: "unknown", value: "y"), { "x" => "y" })).to be false
      end
    end
  end
end
