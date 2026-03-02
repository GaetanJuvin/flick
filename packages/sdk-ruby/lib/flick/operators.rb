# frozen_string_literal: true

module Flick
  module Operators
    module_function

    def match_rule(rule, attributes)
      attr_value = attributes[rule.attribute]
      return false if attr_value.nil?

      case rule.operator
      when "eq"
        attr_value.to_s == rule.value.to_s
      when "neq"
        attr_value.to_s != rule.value.to_s
      when "in"
        return false unless rule.value.is_a?(Array)

        rule.value.map(&:to_s).include?(attr_value.to_s)
      when "not_in"
        return true unless rule.value.is_a?(Array)

        !rule.value.map(&:to_s).include?(attr_value.to_s)
      when "contains"
        attr_value.to_s.include?(rule.value.to_s)
      when "starts_with"
        attr_value.to_s.start_with?(rule.value.to_s)
      when "ends_with"
        attr_value.to_s.end_with?(rule.value.to_s)
      when "gt"
        to_number(attr_value) > to_number(rule.value)
      when "gte"
        to_number(attr_value) >= to_number(rule.value)
      when "lt"
        to_number(attr_value) < to_number(rule.value)
      when "lte"
        to_number(attr_value) <= to_number(rule.value)
      when "regex"
        begin
          Regexp.new(rule.value.to_s).match?(attr_value.to_s)
        rescue RegexpError
          false
        end
      else
        false
      end
    end

    def to_number(val)
      Float(val)
    rescue ArgumentError, TypeError
      0.0
    end
  end
end
