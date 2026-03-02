# frozen_string_literal: true

module Flick
  module Context
    module_function

    # Build an evaluation context hash from an actor object.
    # Actor protocol: respond to #flick_id (required), #flick_attributes (optional).
    # Returns { key: String, attributes: Hash }
    def from_actor(actor)
      return { key: "", attributes: {} } if actor.nil?

      unless actor.respond_to?(:flick_id)
        raise ArgumentError, "Actor must respond to #flick_id"
      end

      attributes = if actor.respond_to?(:flick_attributes)
        normalize_attributes(actor.flick_attributes)
      else
        {}
      end

      { key: actor.flick_id.to_s, attributes: attributes }
    end

    def normalize_attributes(attrs)
      return {} unless attrs.is_a?(Hash)

      attrs.each_with_object({}) do |(k, v), result|
        result[k.to_s] = v
      end
    end
  end
end
