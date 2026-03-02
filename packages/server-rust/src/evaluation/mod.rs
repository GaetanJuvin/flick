use serde::{Deserialize, Serialize};

/// MurmurHash3 (32-bit) - deterministic hash for percentage rollouts.
/// Must produce identical results to TS, Kotlin, and Ruby implementations.
pub fn murmurhash3(key: &str, seed: u32) -> u32 {
    let data = key.as_bytes();
    let len = data.len();
    let mut h1 = seed;
    let c1: u32 = 0xcc9e2d51;
    let c2: u32 = 0x1b873593;

    // Process 4-byte chunks
    let mut i = 0;
    while i + 4 <= len {
        let mut k1 = (data[i] as u32)
            | ((data[i + 1] as u32) << 8)
            | ((data[i + 2] as u32) << 16)
            | ((data[i + 3] as u32) << 24);

        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(c2);

        h1 ^= k1;
        h1 = h1.rotate_left(13);
        h1 = h1.wrapping_mul(5).wrapping_add(0xe6546b64);

        i += 4;
    }

    // Process remaining bytes
    let mut k1: u32 = 0;
    let remaining = len & 3;
    if remaining >= 3 {
        k1 ^= (data[i + 2] as u32) << 16;
    }
    if remaining >= 2 {
        k1 ^= (data[i + 1] as u32) << 8;
    }
    if remaining >= 1 {
        k1 ^= data[i] as u32;
        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(c2);
        h1 ^= k1;
    }

    // Finalization
    h1 ^= len as u32;
    h1 ^= h1 >> 16;
    h1 = h1.wrapping_mul(0x85ebca6b);
    h1 ^= h1 >> 13;
    h1 = h1.wrapping_mul(0xc2b2ae35);
    h1 ^= h1 >> 16;

    h1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationContext {
    pub key: String,
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    pub flag_key: String,
    pub enabled: bool,
    pub gate_type: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagConfig {
    pub key: String,
    pub gate_type: String,
    pub enabled: bool,
    pub gate_config: serde_json::Value,
    #[serde(default)]
    pub groups: Vec<FlagGroupConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagGroupConfig {
    pub id: String,
    pub rules: Vec<GroupRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRule {
    pub attribute: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullFlagConfig {
    pub environment: String,
    pub flags: Vec<FlagConfig>,
    pub version: String,
}

fn match_rule(rule: &GroupRule, attributes: &std::collections::HashMap<String, serde_json::Value>) -> bool {
    let attr_value = match attributes.get(&rule.attribute) {
        Some(v) => v,
        None => return false,
    };

    match rule.operator.as_str() {
        "eq" => value_to_string(attr_value) == value_to_string(&rule.value),
        "neq" => value_to_string(attr_value) != value_to_string(&rule.value),
        "in" => {
            if let Some(arr) = rule.value.as_array() {
                let attr_str = value_to_string(attr_value);
                arr.iter().any(|v| value_to_string(v) == attr_str)
            } else {
                false
            }
        }
        "not_in" => {
            if let Some(arr) = rule.value.as_array() {
                let attr_str = value_to_string(attr_value);
                !arr.iter().any(|v| value_to_string(v) == attr_str)
            } else {
                true
            }
        }
        "contains" => {
            value_to_string(attr_value).contains(&value_to_string(&rule.value))
        }
        "starts_with" => {
            value_to_string(attr_value).starts_with(&value_to_string(&rule.value))
        }
        "ends_with" => {
            value_to_string(attr_value).ends_with(&value_to_string(&rule.value))
        }
        "gt" => value_to_f64(attr_value) > value_to_f64(&rule.value),
        "gte" => value_to_f64(attr_value) >= value_to_f64(&rule.value),
        "lt" => value_to_f64(attr_value) < value_to_f64(&rule.value),
        "lte" => value_to_f64(attr_value) <= value_to_f64(&rule.value),
        "regex" => {
            match regex::Regex::new(&value_to_string(&rule.value)) {
                Ok(re) => re.is_match(&value_to_string(attr_value)),
                Err(_) => false,
            }
        }
        _ => false,
    }
}

fn match_group(rules: &[GroupRule], attributes: &std::collections::HashMap<String, serde_json::Value>) -> bool {
    // All rules within a group are ANDed
    rules.iter().all(|rule| match_rule(rule, attributes))
}

/// Evaluate a single flag against a context.
pub fn evaluate_flag(flag: Option<&FlagConfig>, context: &EvaluationContext) -> EvaluationResult {
    let flag = match flag {
        Some(f) => f,
        None => {
            return EvaluationResult {
                flag_key: String::new(),
                enabled: false,
                gate_type: "boolean".to_string(),
                reason: "flag_not_found".to_string(),
            };
        }
    };

    if !flag.enabled {
        return EvaluationResult {
            flag_key: flag.key.clone(),
            enabled: false,
            gate_type: flag.gate_type.clone(),
            reason: "flag_disabled".to_string(),
        };
    }

    match flag.gate_type.as_str() {
        "boolean" => EvaluationResult {
            flag_key: flag.key.clone(),
            enabled: true,
            gate_type: "boolean".to_string(),
            reason: "boolean_on".to_string(),
        },
        "percentage" => {
            let percentage = flag
                .gate_config
                .get("percentage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as u32;
            let hash_input = format!("{}{}", flag.key, context.key);
            let hash = murmurhash3(&hash_input, 0) % 100;
            let enabled = hash < percentage;
            EvaluationResult {
                flag_key: flag.key.clone(),
                enabled,
                gate_type: "percentage".to_string(),
                reason: if enabled {
                    "percentage_match".to_string()
                } else {
                    "percentage_miss".to_string()
                },
            }
        }
        "group" => {
            if flag.groups.is_empty() {
                return EvaluationResult {
                    flag_key: flag.key.clone(),
                    enabled: false,
                    gate_type: "group".to_string(),
                    reason: "group_miss".to_string(),
                };
            }

            // Groups on a flag are ORed — any matching group enables the flag
            let matched = flag
                .groups
                .iter()
                .any(|group| match_group(&group.rules, &context.attributes));

            EvaluationResult {
                flag_key: flag.key.clone(),
                enabled: matched,
                gate_type: "group".to_string(),
                reason: if matched {
                    "group_match".to_string()
                } else {
                    "group_miss".to_string()
                },
            }
        }
        _ => EvaluationResult {
            flag_key: flag.key.clone(),
            enabled: false,
            gate_type: flag.gate_type.clone(),
            reason: "flag_not_found".to_string(),
        },
    }
}

fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        _ => v.to_string(),
    }
}

fn value_to_f64(v: &serde_json::Value) -> f64 {
    match v {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        serde_json::Value::String(s) => s.parse().unwrap_or(f64::NAN),
        serde_json::Value::Bool(b) => if *b { 1.0 } else { 0.0 },
        _ => f64::NAN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test vectors verified against the TS implementation (packages/shared/src/evaluation.ts)
    #[test]
    fn test_murmurhash3_vectors() {
        assert_eq!(murmurhash3("", 0), 0);
        assert_eq!(murmurhash3("hello", 0), 613153351);
        assert_eq!(murmurhash3("hello", 1), 3142237357);
        assert_eq!(murmurhash3("test", 0), 3127628307);
        assert_eq!(murmurhash3("a", 0), 1009084850);
        assert_eq!(murmurhash3("ab", 0), 2613040991);
        assert_eq!(murmurhash3("abc", 0), 3017643002);
        assert_eq!(murmurhash3("abcd", 0), 1139631978);
        assert_eq!(murmurhash3("feature_flaguser_123", 0), murmurhash3("feature_flaguser_123", 0));
    }

    #[test]
    fn test_murmurhash3_percentage() {
        // Verify percentage calculation is deterministic
        let hash = murmurhash3("my_flaguser_42", 0) % 100;
        let hash2 = murmurhash3("my_flaguser_42", 0) % 100;
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_evaluate_flag_not_found() {
        let ctx = EvaluationContext {
            key: "user_1".to_string(),
            attributes: Default::default(),
        };
        let result = evaluate_flag(None, &ctx);
        assert!(!result.enabled);
        assert_eq!(result.reason, "flag_not_found");
    }

    #[test]
    fn test_evaluate_flag_disabled() {
        let flag = FlagConfig {
            key: "test".to_string(),
            gate_type: "boolean".to_string(),
            enabled: false,
            gate_config: serde_json::json!({}),
            groups: vec![],
        };
        let ctx = EvaluationContext {
            key: "user_1".to_string(),
            attributes: Default::default(),
        };
        let result = evaluate_flag(Some(&flag), &ctx);
        assert!(!result.enabled);
        assert_eq!(result.reason, "flag_disabled");
    }

    #[test]
    fn test_evaluate_boolean_on() {
        let flag = FlagConfig {
            key: "test".to_string(),
            gate_type: "boolean".to_string(),
            enabled: true,
            gate_config: serde_json::json!({}),
            groups: vec![],
        };
        let ctx = EvaluationContext {
            key: "user_1".to_string(),
            attributes: Default::default(),
        };
        let result = evaluate_flag(Some(&flag), &ctx);
        assert!(result.enabled);
        assert_eq!(result.reason, "boolean_on");
    }

    #[test]
    fn test_evaluate_percentage() {
        let flag = FlagConfig {
            key: "test".to_string(),
            gate_type: "percentage".to_string(),
            enabled: true,
            gate_config: serde_json::json!({ "percentage": 50 }),
            groups: vec![],
        };
        // Check a bunch of users — roughly half should be enabled
        let mut enabled_count = 0;
        for i in 0..1000 {
            let ctx = EvaluationContext {
                key: format!("user_{}", i),
                attributes: Default::default(),
            };
            if evaluate_flag(Some(&flag), &ctx).enabled {
                enabled_count += 1;
            }
        }
        // Should be roughly 50% (within 10% tolerance)
        assert!(enabled_count > 400 && enabled_count < 600,
            "Expected ~500 enabled, got {}", enabled_count);
    }

    #[test]
    fn test_match_rule_operators() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("email".to_string(), serde_json::json!("test@example.com"));
        attrs.insert("age".to_string(), serde_json::json!(25));
        attrs.insert("name".to_string(), serde_json::json!("Alice"));

        // eq
        assert!(match_rule(&GroupRule {
            attribute: "name".to_string(),
            operator: "eq".to_string(),
            value: serde_json::json!("Alice"),
        }, &attrs));

        // neq
        assert!(match_rule(&GroupRule {
            attribute: "name".to_string(),
            operator: "neq".to_string(),
            value: serde_json::json!("Bob"),
        }, &attrs));

        // in
        assert!(match_rule(&GroupRule {
            attribute: "name".to_string(),
            operator: "in".to_string(),
            value: serde_json::json!(["Alice", "Bob"]),
        }, &attrs));

        // not_in
        assert!(match_rule(&GroupRule {
            attribute: "name".to_string(),
            operator: "not_in".to_string(),
            value: serde_json::json!(["Bob", "Charlie"]),
        }, &attrs));

        // contains
        assert!(match_rule(&GroupRule {
            attribute: "email".to_string(),
            operator: "contains".to_string(),
            value: serde_json::json!("@example"),
        }, &attrs));

        // starts_with
        assert!(match_rule(&GroupRule {
            attribute: "email".to_string(),
            operator: "starts_with".to_string(),
            value: serde_json::json!("test@"),
        }, &attrs));

        // ends_with
        assert!(match_rule(&GroupRule {
            attribute: "email".to_string(),
            operator: "ends_with".to_string(),
            value: serde_json::json!(".com"),
        }, &attrs));

        // gt
        assert!(match_rule(&GroupRule {
            attribute: "age".to_string(),
            operator: "gt".to_string(),
            value: serde_json::json!(20),
        }, &attrs));

        // gte
        assert!(match_rule(&GroupRule {
            attribute: "age".to_string(),
            operator: "gte".to_string(),
            value: serde_json::json!(25),
        }, &attrs));

        // lt
        assert!(match_rule(&GroupRule {
            attribute: "age".to_string(),
            operator: "lt".to_string(),
            value: serde_json::json!(30),
        }, &attrs));

        // lte
        assert!(match_rule(&GroupRule {
            attribute: "age".to_string(),
            operator: "lte".to_string(),
            value: serde_json::json!(25),
        }, &attrs));

        // regex
        assert!(match_rule(&GroupRule {
            attribute: "email".to_string(),
            operator: "regex".to_string(),
            value: serde_json::json!("^test@.*\\.com$"),
        }, &attrs));
    }
}
