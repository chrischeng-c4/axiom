// PEP 753 — Well-Known Project URL labels (Tick 122).
//
// pyproject.toml `[project.urls]` is a free-form dict whose keys are
// human-readable labels like `Homepage`, `Source Code`, `Bug Tracker`,
// `Documentation`, `Changelog`, etc. PEP 753 standardizes a normalized
// label-matching algorithm so that PyPI, pip, uv, and IDE tooling can
// recognize semantically equivalent labels (e.g. `homepage`, `Home Page`,
// `HOME-PAGE`, `home_page` all collapse to the same canonical
// `Homepage` label) and display the right icon / route the right link.
//
// Normalization (PEP 753):
//   1. Lowercase the label.
//   2. Remove all characters that are NOT ASCII alphanumeric. This
//      drops spaces, hyphens, underscores, and any punctuation.
//   3. Compare the normalized string against the canonical lookup
//      table below.
//
// Canonical labels (PEP 753 "Well-Known Labels"):
//   homepage      -> Homepage
//   source        -> Source
//   download      -> Download
//   changelog     -> Changelog
//   releasenotes  -> Release notes
//   documentation -> Documentation
//   issues        -> Issues
//   funding       -> Funding
//
// This module is the structural primitive. Wiring it into the
// `pep621::ProjectTable` typed reader is a separate tick — the
// `[project.urls]` table belongs to PEP 621 / 753 / 770 dispatch and
// gets its own typed surface there.

use std::collections::BTreeMap;

/// PEP 753 well-known label classification.
///
/// `Other(_)` preserves the verbatim original label so downstream
/// renderers (pip show, uv pkg info, pypi.org) can still display
/// non-standard rows without losing data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WellKnownLabel {
    Homepage,
    Source,
    Download,
    Changelog,
    ReleaseNotes,
    Documentation,
    Issues,
    Funding,
    /// Anything that did not match a canonical entry. The original
    /// label (pre-normalization) is preserved verbatim.
    Other(String),
}

impl WellKnownLabel {
    /// Stable canonical display string. For `Other(s)`, returns the
    /// original label unchanged.
    pub fn canonical(&self) -> &str {
        match self {
            WellKnownLabel::Homepage => "Homepage",
            WellKnownLabel::Source => "Source",
            WellKnownLabel::Download => "Download",
            WellKnownLabel::Changelog => "Changelog",
            WellKnownLabel::ReleaseNotes => "Release notes",
            WellKnownLabel::Documentation => "Documentation",
            WellKnownLabel::Issues => "Issues",
            WellKnownLabel::Funding => "Funding",
            WellKnownLabel::Other(s) => s.as_str(),
        }
    }

    /// True when this label is one of the PEP 753 well-known entries.
    pub fn is_well_known(&self) -> bool {
        !matches!(self, WellKnownLabel::Other(_))
    }
}

/// Apply the PEP 753 normalization (lowercase + strip non-alphanumeric).
/// Exposed publicly so callers building their own URL-table indices
/// can produce the same canonical lookup key.
pub fn normalize_label(label: &str) -> String {
    let mut out = String::with_capacity(label.len());
    for c in label.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        }
    }
    out
}

/// Classify a single label per PEP 753.
pub fn classify(label: &str) -> WellKnownLabel {
    match normalize_label(label).as_str() {
        "homepage" | "home" => WellKnownLabel::Homepage,
        "source" | "sourcecode" | "repository" | "repo" => WellKnownLabel::Source,
        "download" | "downloads" => WellKnownLabel::Download,
        "changelog" | "changes" | "history" => WellKnownLabel::Changelog,
        "releasenotes" => WellKnownLabel::ReleaseNotes,
        "documentation" | "docs" => WellKnownLabel::Documentation,
        "issues" | "issuetracker" | "bugtracker" | "bugs" | "tracker" => WellKnownLabel::Issues,
        "funding" | "sponsor" | "donate" => WellKnownLabel::Funding,
        _ => WellKnownLabel::Other(label.to_string()),
    }
}

/// Classify every entry of a `[project.urls]` table. Returns a triple
/// `(label, original_key, url)` per entry, in iteration order of the
/// input map (`BTreeMap` already gives stable alphabetical order).
///
/// Duplicate classifications (two non-canonical labels collapsing to
/// the same well-known kind, e.g. both `Home Page` and `homepage` in
/// the same table) are preserved as separate entries — PEP 753 is
/// a classifier, not a deduplicator. Downstream tools decide whether
/// to fold or warn on duplicates.
pub fn classify_urls(urls: &BTreeMap<String, String>) -> Vec<(WellKnownLabel, String, String)> {
    urls.iter()
        .map(|(k, v)| (classify(k), k.clone(), v.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map<const N: usize>(pairs: [(&str, &str); N]) -> BTreeMap<String, String> {
        pairs
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn normalize_strips_separators_and_lowers() {
        assert_eq!(normalize_label("Home Page"), "homepage");
        assert_eq!(normalize_label("HOME-PAGE"), "homepage");
        assert_eq!(normalize_label("home_page"), "homepage");
        assert_eq!(normalize_label("home.page"), "homepage");
        assert_eq!(normalize_label("Release Notes"), "releasenotes");
        assert_eq!(normalize_label("Release-Notes"), "releasenotes");
    }

    #[test]
    fn normalize_keeps_alphanumerics() {
        assert_eq!(normalize_label("v2 Changelog"), "v2changelog");
        assert_eq!(normalize_label("3rd-Party"), "3rdparty");
    }

    #[test]
    fn classifies_homepage_variants() {
        assert_eq!(classify("Homepage"), WellKnownLabel::Homepage);
        assert_eq!(classify("HOMEPAGE"), WellKnownLabel::Homepage);
        assert_eq!(classify("Home Page"), WellKnownLabel::Homepage);
        assert_eq!(classify("home-page"), WellKnownLabel::Homepage);
        assert_eq!(classify("home_page"), WellKnownLabel::Homepage);
        assert_eq!(classify("Home"), WellKnownLabel::Homepage);
    }

    #[test]
    fn classifies_source_variants() {
        assert_eq!(classify("Source"), WellKnownLabel::Source);
        assert_eq!(classify("Source Code"), WellKnownLabel::Source);
        assert_eq!(classify("source-code"), WellKnownLabel::Source);
        assert_eq!(classify("Repository"), WellKnownLabel::Source);
        assert_eq!(classify("Repo"), WellKnownLabel::Source);
    }

    #[test]
    fn classifies_download_variants() {
        assert_eq!(classify("Download"), WellKnownLabel::Download);
        assert_eq!(classify("Downloads"), WellKnownLabel::Download);
    }

    #[test]
    fn classifies_changelog_variants() {
        assert_eq!(classify("Changelog"), WellKnownLabel::Changelog);
        assert_eq!(classify("Changes"), WellKnownLabel::Changelog);
        assert_eq!(classify("History"), WellKnownLabel::Changelog);
    }

    #[test]
    fn classifies_release_notes_variants() {
        assert_eq!(classify("Release notes"), WellKnownLabel::ReleaseNotes);
        assert_eq!(classify("Release-Notes"), WellKnownLabel::ReleaseNotes);
        assert_eq!(classify("release_notes"), WellKnownLabel::ReleaseNotes);
    }

    #[test]
    fn classifies_documentation_variants() {
        assert_eq!(classify("Documentation"), WellKnownLabel::Documentation);
        assert_eq!(classify("Docs"), WellKnownLabel::Documentation);
        assert_eq!(classify("DOCS"), WellKnownLabel::Documentation);
    }

    #[test]
    fn classifies_issues_variants() {
        assert_eq!(classify("Issues"), WellKnownLabel::Issues);
        assert_eq!(classify("Bug Tracker"), WellKnownLabel::Issues);
        assert_eq!(classify("issue_tracker"), WellKnownLabel::Issues);
        assert_eq!(classify("bugs"), WellKnownLabel::Issues);
        assert_eq!(classify("Tracker"), WellKnownLabel::Issues);
    }

    #[test]
    fn classifies_funding_variants() {
        assert_eq!(classify("Funding"), WellKnownLabel::Funding);
        assert_eq!(classify("Sponsor"), WellKnownLabel::Funding);
        assert_eq!(classify("Donate"), WellKnownLabel::Funding);
    }

    #[test]
    fn unknown_label_becomes_other_with_verbatim_text() {
        let c = classify("Slack");
        match c {
            WellKnownLabel::Other(s) => assert_eq!(s, "Slack"),
            _ => panic!("expected Other"),
        }
    }

    #[test]
    fn other_preserves_case() {
        let c = classify("CustomLink");
        assert_eq!(c.canonical(), "CustomLink");
        assert!(!c.is_well_known());
    }

    #[test]
    fn well_known_canonical_strings_are_stable() {
        assert_eq!(WellKnownLabel::Homepage.canonical(), "Homepage");
        assert_eq!(WellKnownLabel::Source.canonical(), "Source");
        assert_eq!(WellKnownLabel::Download.canonical(), "Download");
        assert_eq!(WellKnownLabel::Changelog.canonical(), "Changelog");
        assert_eq!(WellKnownLabel::ReleaseNotes.canonical(), "Release notes");
        assert_eq!(WellKnownLabel::Documentation.canonical(), "Documentation");
        assert_eq!(WellKnownLabel::Issues.canonical(), "Issues");
        assert_eq!(WellKnownLabel::Funding.canonical(), "Funding");
    }

    #[test]
    fn is_well_known_returns_correctly() {
        assert!(WellKnownLabel::Homepage.is_well_known());
        assert!(WellKnownLabel::Documentation.is_well_known());
        assert!(!WellKnownLabel::Other("Slack".to_string()).is_well_known());
    }

    #[test]
    fn classify_urls_preserves_order_and_original_keys() {
        let urls = map([
            ("Homepage", "https://example.com"),
            ("Source Code", "https://github.com/foo/bar"),
            ("Bug Tracker", "https://github.com/foo/bar/issues"),
            ("Slack", "https://example.slack.com"),
        ]);
        let classified = classify_urls(&urls);
        assert_eq!(classified.len(), 4);
        // BTreeMap order is alphabetical by key.
        assert_eq!(classified[0].0, WellKnownLabel::Issues); // Bug Tracker
        assert_eq!(classified[0].1, "Bug Tracker");
        assert_eq!(classified[0].2, "https://github.com/foo/bar/issues");
        assert_eq!(classified[1].0, WellKnownLabel::Homepage);
        assert_eq!(classified[1].1, "Homepage");
        assert_eq!(classified[2].0, WellKnownLabel::Other("Slack".to_string()));
        assert_eq!(classified[2].1, "Slack");
        assert_eq!(classified[3].0, WellKnownLabel::Source); // Source Code
    }

    #[test]
    fn empty_url_table_classifies_to_empty_vec() {
        let urls: BTreeMap<String, String> = BTreeMap::new();
        assert!(classify_urls(&urls).is_empty());
    }

    #[test]
    fn classify_urls_keeps_duplicates_distinct() {
        // Two labels that both normalize to "homepage" — PEP 753 says
        // classifier; not deduplicator. Both entries survive.
        let urls = map([("Homepage", "https://a"), ("Home Page", "https://b")]);
        let classified = classify_urls(&urls);
        assert_eq!(classified.len(), 2);
        assert_eq!(classified[0].0, WellKnownLabel::Homepage);
        assert_eq!(classified[1].0, WellKnownLabel::Homepage);
        // Original labels are preserved verbatim so downstream tools
        // can decide whether to fold or warn.
        assert_ne!(classified[0].1, classified[1].1);
    }

    #[test]
    fn realistic_pyproject_urls_table_classifies_cleanly() {
        // From a real-world PyPI project pyproject.toml.
        let urls = map([
            ("Homepage", "https://flask.palletsprojects.com/"),
            ("Documentation", "https://flask.palletsprojects.com/"),
            ("Source", "https://github.com/pallets/flask/"),
            ("Changes", "https://flask.palletsprojects.com/changes/"),
            ("Chat", "https://discord.gg/pallets"),
            ("Donate", "https://palletsprojects.com/donate"),
            ("Issue Tracker", "https://github.com/pallets/flask/issues/"),
        ]);
        let classified = classify_urls(&urls);
        // Count well-known matches.
        let well_known_count = classified
            .iter()
            .filter(|(l, _, _)| l.is_well_known())
            .count();
        // All except "Chat" should be well-known.
        assert_eq!(well_known_count, 6);
        // Chat is the lone Other.
        let chat = classified.iter().find(|(_, k, _)| k == "Chat").unwrap();
        assert_eq!(chat.0, WellKnownLabel::Other("Chat".to_string()));
    }
}
