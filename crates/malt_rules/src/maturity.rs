//! Rule maturity (plan step 6.3): which checks are stable and which are
//! experimental.
//!
//! Metadata only. It does not change what a check reports: the checks it marks
//! experimental are already off under the default `mathworks` preset, and run
//! with `[lint] experimental = true` (see `malt_core::Config::experimental`),
//! `--preset all`, or an explicit rule setting.
//!
//! The classification is one reviewable table, `data/maturity.toml`, with a
//! reason for each entry. It is read here, and by `docs/scripts/gen_rules_docs.ts`
//! for the Maturity columns of the generated docs. Everything not listed is
//! stable. `docs/VERIFY.md` ("Rule maturity (plan 6.3)") has the evidence.

use std::collections::HashMap;
use std::sync::OnceLock;

use malt_core::Maturity;
use serde::Deserialize;

const TABLE: &str = include_str!("data/maturity.toml");
const COMPAT: &str = include_str!("data/compatibility.toml");

/// One entry of `data/maturity.toml`: a selector and why it is experimental.
#[derive(Debug, Deserialize)]
struct Entry {
    engine: Option<String>,
    #[serde(default)]
    ids: Vec<String>,
    prefix: Option<String>,
    #[serde(default)]
    except: Vec<String>,
    compat: Option<String>,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct TableFile {
    experimental: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct CompatFile {
    checks: Vec<CompatEntry>,
}

/// The fields of a compatibility entry that the classification reads.
#[derive(Debug, Deserialize)]
struct CompatEntry {
    id: Option<String>,
    pattern: Option<String>,
    #[serde(default)]
    low_reliability: bool,
}

struct Table {
    entries: Vec<Entry>,
    /// The compatibility entries per check ID: the reason of the first entry
    /// selector that applies to every entry of the ID, if any.
    compat: HashMap<String, usize>,
}

static CLASSIFICATION: OnceLock<Table> = OnceLock::new();

fn table() -> &'static Table {
    CLASSIFICATION.get_or_init(|| {
        let file: TableFile = toml::from_str(TABLE).expect("data/maturity.toml must parse");
        let compat: CompatFile =
            toml::from_str(COMPAT).expect("data/compatibility.toml must parse");
        let selects = |entry: &CompatEntry, key: &str| match key {
            "low_reliability" => entry.low_reliability,
            other => entry.pattern.as_deref() == Some(other),
        };
        let mut by_id: HashMap<&str, Vec<&CompatEntry>> = HashMap::new();
        for entry in &compat.checks {
            if let Some(id) = &entry.id {
                by_id.entry(id).or_default().push(entry);
            }
        }
        let mut flagged = HashMap::new();
        for (id, entries) in by_id {
            // An ID is experimental only when every one of its entries is.
            let reason = file.experimental.iter().position(|e| {
                e.compat
                    .as_deref()
                    .is_some_and(|key| entries.iter().all(|entry| selects(entry, key)))
            });
            if let Some(reason) = reason {
                flagged.insert(id.to_string(), reason);
            }
        }
        Table {
            entries: file.experimental,
            compat: flagged,
        }
    })
}

/// The maturity of an engine: [`Maturity::Experimental`] when the whole engine
/// is listed (e.g. `CODEGEN_ENGINE`). An engine can be stable and still emit
/// experimental checks; ask [`check_maturity`] for those.
pub fn engine_maturity(engine_id: &str) -> Maturity {
    if table()
        .entries
        .iter()
        .any(|e| e.engine.as_deref() == Some(engine_id))
    {
        Maturity::Experimental
    } else {
        Maturity::Stable
    }
}

/// The maturity of a check ID, from the `ids`, `prefix` and `compat` entries of
/// the table. It does not know which engine emits the ID, so the checks of an
/// experimental engine need [`engine_maturity`] as well.
pub fn check_maturity(check_id: &str) -> Maturity {
    experimental_reason(check_id).map_or(Maturity::Stable, |_| Maturity::Experimental)
}

/// Why a check ID is experimental, or `None` when it is stable (or only its
/// engine is experimental).
pub fn experimental_reason(check_id: &str) -> Option<&'static str> {
    let table = table();
    if let Some(&idx) = table.compat.get(check_id) {
        return Some(table.entries[idx].reason.as_str());
    }
    table.entries.iter().find_map(|e| {
        let listed = e.ids.iter().any(|id| id == check_id);
        let prefixed = e
            .prefix
            .as_deref()
            .is_some_and(|p| check_id.starts_with(p) && !e.except.iter().any(|id| id == check_id));
        (listed || prefixed).then_some(e.reason.as_str())
    })
}

/// The engines the table lists as experimental as a whole.
pub fn experimental_engines() -> impl Iterator<Item = &'static str> {
    table().entries.iter().filter_map(|e| e.engine.as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{active_rules, all_rules, registrations};
    use malt_core::Config;

    #[test]
    fn the_table_parses_and_every_entry_has_one_selector_and_a_reason() {
        for entry in &table().entries {
            let selectors = usize::from(entry.engine.is_some())
                + usize::from(!entry.ids.is_empty())
                + usize::from(entry.prefix.is_some())
                + usize::from(entry.compat.is_some());
            assert_eq!(selectors, 1, "one selector per entry: {}", entry.reason);
            assert!(entry.reason.len() > 20, "a real reason: {}", entry.reason);
            assert!(entry.except.is_empty() || entry.prefix.is_some());
        }
    }

    #[test]
    fn engines_in_the_table_are_registered() {
        let registered: Vec<&str> = registrations().iter().map(|r| r.id).collect();
        for engine in experimental_engines() {
            assert!(registered.contains(&engine), "{engine} is not an engine");
        }
    }

    #[test]
    fn classification_examples() {
        assert_eq!(engine_maturity("CODEGEN_ENGINE"), Maturity::Experimental);
        assert_eq!(engine_maturity("BUGS_ENGINE"), Maturity::Stable);
        assert_eq!(check_maturity("ERTAG"), Maturity::Experimental);
        assert_eq!(check_maturity("NOSEMI"), Maturity::Stable);
        assert_eq!(check_maturity("naming.variable.casing"), Maturity::Stable);
        assert_eq!(
            check_maturity("naming.class.casing"),
            Maturity::Experimental
        );
        assert_eq!(check_maturity("FCNLL"), Maturity::Stable);
        assert_eq!(check_maturity("FCNIL"), Maturity::Experimental);
        assert!(experimental_reason("FCLEN").is_some());
    }

    #[test]
    fn compat_experimental_count_covers_the_flagged_entries() {
        // Low-reliability, `object` and `unverified` entries, plus the gates; the
        // docs generator counts the same IDs (`--maturity-report`).
        let compat: CompatFile = toml::from_str(COMPAT).unwrap();
        let experimental = compat
            .checks
            .iter()
            .filter_map(|e| e.id.as_deref())
            .filter(|id| check_maturity(id) == Maturity::Experimental)
            .count();
        assert!(experimental >= 897 + 183, "got {experimental}");
    }

    #[test]
    fn compat_flags_select_entries() {
        let compat: CompatFile = toml::from_str(COMPAT).unwrap();
        let low = compat
            .checks
            .iter()
            .filter(|e| e.low_reliability)
            .filter_map(|e| e.id.as_deref())
            .next()
            .expect("the data has low-reliability entries");
        assert_eq!(check_maturity(low), Maturity::Experimental);
    }

    #[test]
    fn experimental_engines_are_off_by_default_and_on_with_the_flag() {
        let ids = |toml: &str| -> Vec<String> {
            let config = Config::from_toml(toml).unwrap();
            active_rules(&config)
                .iter()
                .map(|r| r.id().to_string())
                .collect()
        };
        let default = ids("");
        let flagged = ids("[lint]\nexperimental = true\n");
        for engine in experimental_engines() {
            assert!(
                !default.contains(&engine.to_string()),
                "{engine} on by default"
            );
            assert!(
                flagged.contains(&engine.to_string()),
                "{engine} off with the flag"
            );
        }
        // The flag lifts the maturity gate only: categories the preset disables stay off.
        assert!(!flagged.contains(&"NAMING_ENGINE".to_string()));
        assert!(!flagged.contains(&"CUSTOM_CHECKS".to_string()));
    }

    #[test]
    fn explicit_rule_settings_win_over_the_experimental_gate() {
        let on = Config::from_toml("[lint.rules.CODEGEN_ENGINE]\nseverity = \"error\"\n").unwrap();
        assert!(active_rules(&on).iter().any(|r| r.id() == "CODEGEN_ENGINE"));
        let off = Config::from_toml(
            "[lint]\nexperimental = true\n[lint.rules]\nCODEGEN_ENGINE = \"off\"\n",
        )
        .unwrap();
        assert!(!active_rules(&off)
            .iter()
            .any(|r| r.id() == "CODEGEN_ENGINE"));
    }

    /// Every ID the default analysis reports on a probe that exercises many
    /// engines is stable, and the experimental flag adds findings only.
    #[test]
    fn default_output_has_no_experimental_check() {
        let source = "function f(x)\nerror('plain message');\ntry\n x = 1;\ncatch\nend\nif size(x,1) == 1, end\nsum(x(:) > 3);\nend\n";
        let ids = |toml: &str| -> Vec<&'static str> {
            let config = Config::from_toml(toml).unwrap();
            let registry = malt_core::RuleRegistry::new(active_rules(&config), &config);
            let mut linter = malt_core::Linter::new(registry);
            let mut found: Vec<_> = linter
                .lint(source, std::path::Path::new("probe.m"))
                .unwrap()
                .iter()
                .map(|d| d.rule_id)
                .collect();
            found.sort_unstable();
            found.dedup();
            found
        };
        let default = ids("");
        for id in &default {
            assert_eq!(check_maturity(id), Maturity::Stable, "{id} is experimental");
        }
        let flagged = ids("[lint]\nexperimental = true\n");
        for id in &default {
            assert!(flagged.contains(id), "{id} lost with the flag");
        }
        let extra: Vec<_> = flagged.iter().filter(|id| !default.contains(id)).collect();
        assert!(!extra.is_empty(), "the flag turned nothing on");
        for id in extra {
            assert_eq!(check_maturity(id), Maturity::Experimental, "{id}");
        }
        // `preset = "all"` is unaffected by the flag.
        assert_eq!(
            ids("[lint]\npreset = \"all\"\n"),
            ids("[lint]\npreset = \"all\"\nexperimental = true\n")
        );
        assert!(all_rules(&Config::default()).len() >= 19);
    }
}
