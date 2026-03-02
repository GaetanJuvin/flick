# frozen_string_literal: true

module Flick
  module Evaluator
    module_function

    def evaluate(flag, context)
      unless flag
        return EvaluationResult.new(
          flag_key: "",
          enabled: false,
          gate_type: "boolean",
          reason: "flag_not_found"
        )
      end

      unless flag.enabled
        return EvaluationResult.new(
          flag_key: flag.key,
          enabled: false,
          gate_type: flag.gate_type,
          reason: "flag_disabled"
        )
      end

      case flag.gate_type
      when "boolean"
        EvaluationResult.new(
          flag_key: flag.key,
          enabled: true,
          gate_type: "boolean",
          reason: "boolean_on"
        )
      when "percentage"
        evaluate_percentage(flag, context)
      when "group"
        evaluate_group(flag, context)
      else
        EvaluationResult.new(
          flag_key: flag.key,
          enabled: false,
          gate_type: flag.gate_type,
          reason: "flag_not_found"
        )
      end
    end

    def evaluate_percentage(flag, context)
      percentage = flag.gate_config["percentage"] || 0
      hash = MurmurHash3.hash("#{flag.key}#{context[:key]}") % 100
      enabled = hash < percentage

      EvaluationResult.new(
        flag_key: flag.key,
        enabled: enabled,
        gate_type: "percentage",
        reason: enabled ? "percentage_match" : "percentage_miss"
      )
    end

    def evaluate_group(flag, context)
      groups = flag.groups || []

      if groups.empty?
        return EvaluationResult.new(
          flag_key: flag.key,
          enabled: false,
          gate_type: "group",
          reason: "group_miss"
        )
      end

      attributes = context[:attributes] || {}
      matched = groups.any? { |group| match_group(group.rules, attributes) }

      EvaluationResult.new(
        flag_key: flag.key,
        enabled: matched,
        gate_type: "group",
        reason: matched ? "group_match" : "group_miss"
      )
    end

    def match_group(rules, attributes)
      rules.all? { |rule| Operators.match_rule(rule, attributes) }
    end
  end
end
