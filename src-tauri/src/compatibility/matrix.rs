use serde::Deserialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityDecision {
    FullySupported,
    PauseLive,
    PauseLiveAndImport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveParserDecision {
    Verified,
    ProbeRequired,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionSet {
    pub game_version: String,
    pub companion_version: String,
    pub event_schema_version: u16,
}

impl VersionSet {
    pub fn new(
        game_version: impl Into<String>,
        companion_version: impl Into<String>,
        event_schema_version: u16,
    ) -> Self {
        Self {
            game_version: game_version.into(),
            companion_version: companion_version.into(),
            event_schema_version,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CompatibilityMatrix {
    records: Vec<CompatibilityRecord>,
}

#[derive(Clone, Debug, Deserialize)]
struct CompatibilityRecord {
    game_version: String,
    event_schema_versions: Vec<u16>,
    companion_versions: Vec<String>,
    #[allow(dead_code)] // Retained for the read-only parser contract consumed by a later task.
    save_parser_versions: Vec<u16>,
    #[allow(dead_code)] // Retained for the local catalog contract consumed by a later task.
    catalog_parser_versions: Vec<u16>,
    probe_required: bool,
}

impl CompatibilityMatrix {
    pub fn embedded() -> Result<Self, serde_json::Error> {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/compatibility.json"
        )))
    }

    pub fn fixture_for(game_version: impl Into<String>) -> Self {
        Self {
            records: vec![CompatibilityRecord {
                game_version: game_version.into(),
                event_schema_versions: vec![1],
                companion_versions: vec!["0.1.x".to_owned()],
                save_parser_versions: vec![1],
                catalog_parser_versions: vec![1],
                probe_required: true,
            }],
        }
    }

    pub fn decision(&self, versions: &VersionSet) -> CompatibilityDecision {
        let Some(record) = self
            .records
            .iter()
            .find(|record| record.game_version == versions.game_version)
        else {
            return CompatibilityDecision::PauseLiveAndImport;
        };

        if !record
            .event_schema_versions
            .contains(&versions.event_schema_version)
        {
            return CompatibilityDecision::PauseLiveAndImport;
        }

        if !record
            .companion_versions
            .iter()
            .any(|rule| version_matches(rule, &versions.companion_version))
        {
            return CompatibilityDecision::PauseLive;
        }

        if record.probe_required {
            CompatibilityDecision::PauseLive
        } else {
            CompatibilityDecision::FullySupported
        }
    }

    pub fn save_parser_decision(
        &self,
        game_version: &str,
        parser_version: u16,
    ) -> SaveParserDecision {
        let Some(record) = self
            .records
            .iter()
            .find(|record| record.game_version == game_version)
        else {
            return SaveParserDecision::Unsupported;
        };

        if !record.save_parser_versions.contains(&parser_version) {
            return SaveParserDecision::Unsupported;
        }

        if record.probe_required {
            SaveParserDecision::ProbeRequired
        } else {
            SaveParserDecision::Verified
        }
    }

    pub fn catalog_parser_decision(
        &self,
        game_version: &str,
        parser_version: u16,
    ) -> SaveParserDecision {
        let Some(record) = self
            .records
            .iter()
            .find(|record| record.game_version == game_version)
        else {
            return SaveParserDecision::Unsupported;
        };

        if !record.catalog_parser_versions.contains(&parser_version) {
            return SaveParserDecision::Unsupported;
        }

        if record.probe_required {
            SaveParserDecision::ProbeRequired
        } else {
            SaveParserDecision::Verified
        }
    }
}

fn version_matches(rule: &str, version: &str) -> bool {
    match rule.strip_suffix(".x") {
        Some(prefix) => version
            .strip_prefix(&format!("{prefix}."))
            .is_some_and(|patch| {
                !patch.is_empty() && patch.bytes().all(|byte| byte.is_ascii_digit())
            }),
        None => rule == version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_parser_is_probe_gated_until_explicitly_approved() {
        let matrix = CompatibilityMatrix::fixture_for("synthetic-1");
        assert_eq!(
            matrix.catalog_parser_decision("synthetic-1", 1),
            SaveParserDecision::ProbeRequired
        );
        assert_eq!(
            matrix.catalog_parser_decision("unknown", 1),
            SaveParserDecision::Unsupported
        );
    }

    #[test]
    fn embedded_1_0_5_approval_allows_live_events_but_not_unverified_save_imports() {
        let matrix = CompatibilityMatrix::embedded().unwrap();

        assert_eq!(
            matrix.decision(&VersionSet::new("1.0.5", "0.1.3", 1)),
            CompatibilityDecision::FullySupported
        );
        assert_eq!(
            matrix.catalog_parser_decision(
                "catalog-sha256:18a3827b479958c768c2b18f565c8081460bea3e055b78305e9f7088aa46c304",
                1,
            ),
            SaveParserDecision::Verified
        );
        assert_eq!(
            matrix.save_parser_decision("1.0.5", 1),
            SaveParserDecision::Unsupported
        );
    }
}
