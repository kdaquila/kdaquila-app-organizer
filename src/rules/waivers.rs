//! The rules waived for one path, and the resulting active/inactive answer.

use super::Rule;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default)]
pub struct Waivers(pub BTreeSet<Rule>);

impl Waivers {
    /// A rule is active unless it was waived directly, or transitively lost
    /// what it depends on.
    pub fn active(&self, rule: Rule) -> bool {
        if self.0.contains(&rule) {
            return false;
        }
        match rule.depends_on() {
            Some(dep) => self.active(dep),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn waiving(rules: &[Rule]) -> Waivers {
        Waivers(rules.iter().copied().collect())
    }

    #[test]
    fn waiving_a_rule_deactivates_what_depends_on_it() {
        // How `tests/**` becomes free: with no single primary export, there is
        // nothing to derive a filename from and nothing to hold to a budget.
        let waivers = waiving(&[Rule::SinglePrimaryExport]);
        assert!(!waivers.active(Rule::FilenameMatchesExport));
        assert!(!waivers.active(Rule::MaxFileLines));
        // But casing has no such dependency and still applies.
        assert!(waivers.active(Rule::NameCasing));
    }

    #[test]
    fn waiving_a_dependant_leaves_its_dependency_alone() {
        // `mod.rs` cannot be named after its export, but it is still held to
        // one export and to the line budget.
        let waivers = waiving(&[Rule::FilenameMatchesExport]);
        assert!(waivers.active(Rule::SinglePrimaryExport));
        assert!(waivers.active(Rule::MaxFileLines));
    }

    #[test]
    fn nothing_is_waived_by_default() {
        let waivers = Waivers::default();
        for rule in Rule::ALL {
            assert!(waivers.active(rule), "{} should be active", rule.as_str());
        }
    }
}
