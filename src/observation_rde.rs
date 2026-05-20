//! Informative mapping from [`MeaningDelta`](crate::semantic_lineage::MeaningDeltaInput) `observation` keys to RDE category hints.
//!
//! Non-normative: hints only; does not auto-populate or validate RDE payloads.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// RDE category keys aligned with Phase 1 interchange (`kotonoha-spec` `rde-review-output`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RdeCategoryKey {
    Preserved,
    Transformed,
    Complemented,
    IntentionallyUnresolved,
    Lost,
    DeviationRisk,
    NextUpdatePolicy,
}

impl RdeCategoryKey {
    pub fn as_str(self) -> &'static str {
        match self {
            RdeCategoryKey::Preserved => "preserved",
            RdeCategoryKey::Transformed => "transformed",
            RdeCategoryKey::Complemented => "complemented",
            RdeCategoryKey::IntentionallyUnresolved => "intentionally_unresolved",
            RdeCategoryKey::Lost => "lost",
            RdeCategoryKey::DeviationRisk => "deviation_risk",
            RdeCategoryKey::NextUpdatePolicy => "next_update_policy",
        }
    }
}

/// One observation key mapped to a target RDE category with extracted hint strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationRdeHint {
    pub observation_key: String,
    pub category: RdeCategoryKey,
    pub hints: Vec<String>,
}

/// Result of [`map_observation_to_rde_hints`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservationRdeHints {
    pub hints: Vec<ObservationRdeHint>,
    pub unknown_keys: Vec<String>,
}

/// Maps `observation` object keys to RDE category **hints** (not auto-filled RDE items).
///
/// | observation key | RDE category |
/// | --- | --- |
/// | `preserved` | `preserved` |
/// | `lost` | `lost` |
/// | `transformed` | `transformed` |
/// | `unresolved`, `intentionally_unresolved` | `intentionally_unresolved` |
pub fn map_observation_to_rde_hints(observation: &Value) -> ObservationRdeHints {
    let Some(obj) = observation.as_object() else {
        return ObservationRdeHints::default();
    };

    let mut hints = Vec::new();
    let mut unknown_keys = Vec::new();

    for (key, val) in obj {
        let normalized = key.as_str();
        match normalized {
            "preserved" => push_hint(&mut hints, key, RdeCategoryKey::Preserved, val),
            "lost" => push_hint(&mut hints, key, RdeCategoryKey::Lost, val),
            "transformed" => push_hint(&mut hints, key, RdeCategoryKey::Transformed, val),
            "unresolved" | "intentionally_unresolved" => {
                push_hint(
                    &mut hints,
                    key,
                    RdeCategoryKey::IntentionallyUnresolved,
                    val,
                );
            }
            _ => unknown_keys.push(key.clone()),
        }
    }

    ObservationRdeHints {
        hints,
        unknown_keys,
    }
}

fn push_hint(out: &mut Vec<ObservationRdeHint>, key: &str, category: RdeCategoryKey, val: &Value) {
    let extracted = extract_hint_strings(val);
    if extracted.is_empty() {
        return;
    }
    out.push(ObservationRdeHint {
        observation_key: key.to_string(),
        category,
        hints: extracted,
    });
}

fn extract_hint_strings(val: &Value) -> Vec<String> {
    match val {
        Value::String(s) if !s.trim().is_empty() => vec![s.clone()],
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| {
                v.as_str()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
            })
            .collect(),
        Value::Object(map) => map
            .get("summary")
            .or_else(|| map.get("label"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(|s| vec![s.to_string()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_observation_yields_no_hints() {
        let r = map_observation_to_rde_hints(&json!({}));
        assert!(r.hints.is_empty());
        assert!(r.unknown_keys.is_empty());
    }

    #[test]
    fn maps_preserved_and_lost_arrays() {
        let r = map_observation_to_rde_hints(&json!({
            "preserved": ["intent", "scope"],
            "lost": ["footnote nuance"]
        }));
        assert_eq!(r.hints.len(), 2);
        let preserved = r
            .hints
            .iter()
            .find(|h| h.category == RdeCategoryKey::Preserved)
            .expect("preserved hint");
        assert_eq!(preserved.hints, vec!["intent", "scope"]);
        let lost = r
            .hints
            .iter()
            .find(|h| h.category == RdeCategoryKey::Lost)
            .expect("lost hint");
        assert_eq!(lost.hints, vec!["footnote nuance"]);
    }

    #[test]
    fn maps_transformed_key() {
        let r = map_observation_to_rde_hints(&json!({ "transformed": "rewritten scope" }));
        assert_eq!(r.hints.len(), 1);
        assert_eq!(r.hints[0].category, RdeCategoryKey::Transformed);
        assert_eq!(r.hints[0].hints, vec!["rewritten scope"]);
    }

    #[test]
    fn unresolved_alias_maps_to_intentionally_unresolved() {
        let r = map_observation_to_rde_hints(&json!({ "unresolved": "open question" }));
        assert_eq!(r.hints.len(), 1);
        assert_eq!(r.hints[0].category, RdeCategoryKey::IntentionallyUnresolved);
        assert_eq!(r.hints[0].hints, vec!["open question"]);
    }

    #[test]
    fn unknown_keys_collected() {
        let r = map_observation_to_rde_hints(&json!({ "custom_field": true }));
        assert!(r.hints.is_empty());
        assert_eq!(r.unknown_keys, vec!["custom_field"]);
    }

    #[test]
    fn non_object_observation_is_empty() {
        let r = map_observation_to_rde_hints(&json!([]));
        assert!(r.hints.is_empty());
    }
}
