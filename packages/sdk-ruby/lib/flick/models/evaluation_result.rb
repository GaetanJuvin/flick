# frozen_string_literal: true

module Flick
  EvaluationResult = Struct.new(:flag_key, :enabled, :gate_type, :reason, keyword_init: true)
end
