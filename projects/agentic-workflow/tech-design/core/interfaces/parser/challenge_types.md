---
id: sdd-parser-challenge-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: core-concept-model-and-invariants
    claim: core-concept-model-and-invariants
    coverage: full
    rationale: "Core model/parser TDs define AW Core domain nouns, invariants, and artifact structure."
---

# ChallengeParser

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/parser/challenge.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ChallengeParser` | projects/agentic-workflow/src/parser/challenge.rs | struct | pub | 114 |  |
| `parse_challenge_verdict` | projects/agentic-workflow/src/parser/challenge.rs | function | pub | 14 | parse_challenge_verdict(path: &Path) -> Result<ChallengeVerdict> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ChallengeParser:
    type: object
    required: []
    description: ChallengeParser.
    properties: {}
    x-rust-struct:
      derive: []
      unit: true
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/parser/challenge.rs -->
```rust
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/challenge_types.md#source
use crate::models::ChallengeVerdict;
use crate::Result;
use std::path::Path;

/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/challenge_types.md#source
/// Parse verdict from CHALLENGE.md (old) or proposal.md (new)
///
/// Supports both formats:
/// - Old: CHALLENGE.md with checkbox verdicts
/// - New: proposal.md with <review> XML blocks
pub fn parse_challenge_verdict(path: &Path) -> Result<ChallengeVerdict> {
    if !path.exists() {
        // Try proposal.md if CHALLENGE.md doesn't exist
        if path.file_name() == Some(std::ffi::OsStr::new("CHALLENGE.md")) {
            let proposal_path = path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid path"))?
                .join("proposal.md");

            if proposal_path.exists() {
                return parse_verdict_from_proposal(&proposal_path);
            }
        }
        anyhow::bail!("Challenge/review file not found");
    }

    let content = std::fs::read_to_string(path)?;

    // Check for XML format
    if content.contains("<review") {
        parse_verdict_from_review_xml(&content)
    } else {
        // Old format: checkbox parsing
        parse_verdict_from_checkboxes(&content)
    }
}

/// Parse verdict from proposal.md containing <review> blocks
fn parse_verdict_from_proposal(proposal_path: &Path) -> Result<ChallengeVerdict> {
    let content = std::fs::read_to_string(proposal_path)?;
    let latest_review = crate::parser::parse_latest_review(&content)?;

    match latest_review {
        Some(review) => Ok(match review.status.to_lowercase().as_str() {
            "approved" => ChallengeVerdict::Approved,
            "needs_revision" => ChallengeVerdict::NeedsRevision,
            "rejected" => ChallengeVerdict::Rejected,
            _ => ChallengeVerdict::Unknown,
        }),
        None => Ok(ChallengeVerdict::Unknown),
    }
}

/// Parse verdict from XML review blocks
fn parse_verdict_from_review_xml(content: &str) -> Result<ChallengeVerdict> {
    let latest_review = crate::parser::parse_latest_review(content)?;

    match latest_review {
        Some(review) => {
            // Check content for verdict keywords
            let content_lower = review.content.to_lowercase();

            if content_lower.contains("verdict") {
                if content_lower.contains("approved") {
                    Ok(ChallengeVerdict::Approved)
                } else if content_lower.contains("needs_revision")
                    || content_lower.contains("needs revision")
                {
                    Ok(ChallengeVerdict::NeedsRevision)
                } else if content_lower.contains("rejected") {
                    Ok(ChallengeVerdict::Rejected)
                } else {
                    // Fall back to status attribute
                    Ok(match review.status.to_lowercase().as_str() {
                        "approved" => ChallengeVerdict::Approved,
                        "needs_revision" => ChallengeVerdict::NeedsRevision,
                        "rejected" => ChallengeVerdict::Rejected,
                        _ => ChallengeVerdict::Unknown,
                    })
                }
            } else {
                Ok(ChallengeVerdict::Unknown)
            }
        }
        None => Ok(ChallengeVerdict::Unknown),
    }
}

/// Parse verdict from old checkbox format
fn parse_verdict_from_checkboxes(content: &str) -> Result<ChallengeVerdict> {
    let content_lower = content.to_lowercase();

    if content_lower.contains("[x] approved") || content_lower.contains("[✓] approved") {
        Ok(ChallengeVerdict::Approved)
    } else if content_lower.contains("[x] needs_revision")
        || content_lower.contains("[x] needs revision")
        || content_lower.contains("[✓] needs_revision")
        || content_lower.contains("[✓] needs revision")
    {
        Ok(ChallengeVerdict::NeedsRevision)
    } else if content_lower.contains("[x] rejected") || content_lower.contains("[✓] rejected") {
        Ok(ChallengeVerdict::Rejected)
    } else {
        Ok(ChallengeVerdict::Unknown)
    }
}

// Challenge parser - placeholder for future detailed parsing
/// ChallengeParser.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/parser/challenge_types.md#schema
pub struct ChallengeParser;
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/parser/challenge.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete challenge parser module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
