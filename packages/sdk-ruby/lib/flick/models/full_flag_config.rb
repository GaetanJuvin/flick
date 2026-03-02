# frozen_string_literal: true

module Flick
  FullFlagConfig = Struct.new(:environment, :flags, :version, keyword_init: true) do
    def self.from_hash(hash)
      new(
        environment: hash["environment"],
        flags: (hash["flags"] || []).map { |f| FlagConfig.from_hash(f) },
        version: hash["version"]
      )
    end
  end
end
