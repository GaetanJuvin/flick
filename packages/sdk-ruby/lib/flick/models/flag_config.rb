# frozen_string_literal: true

module Flick
  GroupRule = Struct.new(:attribute, :operator, :value, keyword_init: true) do
    def self.from_hash(hash)
      new(
        attribute: hash["attribute"],
        operator: hash["operator"],
        value: hash["value"]
      )
    end
  end

  FlagGroup = Struct.new(:id, :rules, keyword_init: true) do
    def self.from_hash(hash)
      new(
        id: hash["id"],
        rules: (hash["rules"] || []).map { |r| GroupRule.from_hash(r) }
      )
    end
  end

  FlagConfig = Struct.new(:key, :gate_type, :enabled, :gate_config, :groups, keyword_init: true) do
    def self.from_hash(hash)
      new(
        key: hash["key"],
        gate_type: hash["gate_type"],
        enabled: hash["enabled"],
        gate_config: hash["gate_config"] || {},
        groups: (hash["groups"] || []).map { |g| FlagGroup.from_hash(g) }
      )
    end
  end
end
