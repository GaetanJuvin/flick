# frozen_string_literal: true

require "spec_helper"

RSpec.describe Flick::Cache do
  subject(:cache) { described_class.new }

  let(:flags) do
    [
      Flick::FlagConfig.new(key: "flag-a", gate_type: "boolean", enabled: true, gate_config: {}, groups: []),
      Flick::FlagConfig.new(key: "flag-b", gate_type: "boolean", enabled: false, gate_config: {}, groups: [])
    ]
  end

  describe "#update" do
    it "stores flags and returns true on first update" do
      expect(cache.update(flags, "v1")).to be true
      expect(cache.get("flag-a").enabled).to be true
      expect(cache.get("flag-b").enabled).to be false
    end

    it "returns false when version matches" do
      cache.update(flags, "v1")
      expect(cache.update(flags, "v1")).to be false
    end

    it "updates when version changes" do
      cache.update(flags, "v1")
      new_flags = [Flick::FlagConfig.new(key: "flag-a", gate_type: "boolean", enabled: false, gate_config: {}, groups: [])]
      expect(cache.update(new_flags, "v2")).to be true
      expect(cache.get("flag-a").enabled).to be false
      expect(cache.get("flag-b")).to be_nil # old flag removed
    end
  end

  describe "#get" do
    it "returns nil for unknown keys" do
      expect(cache.get("missing")).to be_nil
    end
  end

  describe "#get_all" do
    it "returns all flags" do
      cache.update(flags, "v1")
      expect(cache.get_all.map(&:key)).to contain_exactly("flag-a", "flag-b")
    end
  end

  describe "#empty?" do
    it "is true when no flags loaded" do
      expect(cache).to be_empty
    end

    it "is false after update" do
      cache.update(flags, "v1")
      expect(cache).not_to be_empty
    end
  end

  describe "thread safety" do
    it "handles concurrent reads and writes" do
      threads = 10.times.map do |i|
        Thread.new do
          100.times do |j|
            cache.update(flags, "v#{i}-#{j}")
            cache.get("flag-a")
            cache.get_all
          end
        end
      end
      threads.each(&:join) # Should not raise
    end
  end
end
