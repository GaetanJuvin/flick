# frozen_string_literal: true

require "spec_helper"

RSpec.describe Flick::MurmurHash3 do
  # Cross-SDK test vectors generated from the TypeScript implementation.
  # These MUST match across all SDKs for deterministic percentage rollouts.
  VECTORS = [
    { key: "",                          seed: 0, expected: 0 },
    { key: "hello",                     seed: 0, expected: 613153351 },
    { key: "test",                      seed: 0, expected: 3127628307 },
    { key: "flag-key",                  seed: 0, expected: 1386299095 },
    { key: "new-checkoutuser-123",      seed: 0, expected: 2280669634 },
    { key: "premium-featureuser-456",   seed: 0, expected: 1766081557 },
    { key: "a",                         seed: 0, expected: 1009084850 },
    { key: "ab",                        seed: 0, expected: 2613040991 },
    { key: "abc",                       seed: 0, expected: 3017643002 },
    { key: "abcd",                      seed: 0, expected: 1139631978 },
    { key: "abcde",                     seed: 0, expected: 3902511862 },
  ].freeze

  VECTORS.each do |v|
    it "hashes #{v[:key].inspect} to #{v[:expected]}" do
      result = described_class.hash(v[:key], v[:seed])
      expect(result).to eq(v[:expected])
    end
  end

  it "returns unsigned 32-bit integers" do
    100.times do |i|
      result = described_class.hash("test-#{i}")
      expect(result).to be >= 0
      expect(result).to be <= 0xFFFFFFFF
    end
  end

  it "is deterministic" do
    100.times do
      expect(described_class.hash("same-key")).to eq(described_class.hash("same-key"))
    end
  end
end
