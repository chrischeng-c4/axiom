// HANDWRITE-BEGIN gap="missing-generator:ddd-context-map-contract" tracker="#2291" reason="The invariant checker implements DDD identity grammar and relationship direction rules that are not yet expressible by codegen."
//! Invariant checks for the DDD context-map contract.
//!
//! @spec apps/agentic-workflow/tech-design/core/logic/ddd-context-map-contract.md#logic

use super::{DddContextMap, DddIdentity, DddIdentityKind, DddRelationKind, DDD_CONTEXT_MAP_SCHEMA};
use anyhow::{bail, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
struct ParsedIdentity {
    kind: DddIdentityKind,
    context: String,
}

/// Parse and validate one context-map YAML document.
pub fn parse_context_map(yaml: &str) -> Result<DddContextMap> {
    let map = serde_yaml::from_str(yaml).context("parse DDD context-map YAML")?;
    validate_context_map(&map)?;
    Ok(map)
}

/// Validate stable DDD identities and their allowed relationships.
///
/// Projection metadata remains deliberately opaque here: paths and Markdown
/// anchors may change without changing a logical identity or relationship.
pub fn validate_context_map(map: &DddContextMap) -> Result<()> {
    if map.schema_version != DDD_CONTEXT_MAP_SCHEMA {
        bail!(
            "unsupported DDD context-map schema `{}`; expected `{DDD_CONTEXT_MAP_SCHEMA}`",
            map.schema_version
        );
    }
    if map.nodes.is_empty() {
        bail!("DDD context-map requires at least one identity");
    }

    let mut identities = BTreeMap::new();
    let mut parsed = BTreeMap::new();
    for node in &map.nodes {
        let parsed_identity = parse_identity(node)?;
        if identities.insert(node.id.as_str(), node).is_some() {
            bail!("duplicate DDD identity ownership for `{}`", node.id);
        }
        parsed.insert(node.id.as_str(), parsed_identity);
    }

    let contexts = parsed
        .iter()
        .filter_map(|(id, identity)| (identity.kind == DddIdentityKind::Context).then_some(*id))
        .collect::<BTreeSet<_>>();
    for (id, identity) in &parsed {
        if identity.kind != DddIdentityKind::Context {
            let context_id = format!("context:{}", identity.context);
            if !contexts.contains(context_id.as_str()) {
                bail!("DDD identity `{id}` belongs to undeclared bounded context `{context_id}`");
            }
        }
    }

    let mut context_dependencies = BTreeSet::new();
    for relation in &map.relations {
        let (_, from) = lookup_identity(&identities, &parsed, &relation.from, "from")?;
        let (_, to) = lookup_identity(&identities, &parsed, &relation.to, "to")?;
        validate_relation_shape(relation.kind, &relation.from, from, &relation.to, to)?;
        if relation.kind == DddRelationKind::DependsOn {
            context_dependencies.insert((from.context.clone(), to.context.clone()));
        }
    }

    for relation in &map.relations {
        if matches!(
            relation.kind,
            DddRelationKind::Contains | DddRelationKind::DependsOn
        ) {
            continue;
        }
        let (_, from) = lookup_identity(&identities, &parsed, &relation.from, "from")?;
        let (_, to) = lookup_identity(&identities, &parsed, &relation.to, "to")?;
        if from.context != to.context
            && !context_dependencies.contains(&(from.context.clone(), to.context.clone()))
        {
            bail!(
                "cross-context relation `{}` -> `{}` requires declared context dependency context:{} -> context:{}",
                relation.from,
                relation.to,
                from.context,
                to.context
            );
        }
    }

    Ok(())
}

fn parse_identity(node: &DddIdentity) -> Result<ParsedIdentity> {
    let (prefix, body) = node
        .id
        .split_once(':')
        .with_context(|| format!("DDD identity `{}` must have a kind prefix", node.id))?;
    if prefix != node.kind.as_str() {
        bail!(
            "DDD identity `{}` kind prefix `{prefix}` does not match declared kind `{}`",
            node.id,
            node.kind.as_str()
        );
    }

    let parts = body.split('/').collect::<Vec<_>>();
    let expected_parts = if node.kind == DddIdentityKind::Context {
        1
    } else {
        2
    };
    if parts.len() != expected_parts || parts.iter().any(|part| !is_slug(part)) {
        bail!(
            "DDD identity `{}` must use {} lowercase kebab-case segment(s), never a path or Markdown anchor",
            node.id,
            expected_parts
        );
    }

    Ok(ParsedIdentity {
        kind: node.kind,
        context: parts[0].to_string(),
    })
}

fn is_slug(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes[0].is_ascii_lowercase()
        && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn lookup_identity<'a>(
    identities: &'a BTreeMap<&str, &'a DddIdentity>,
    parsed: &'a BTreeMap<&str, ParsedIdentity>,
    id: &str,
    role: &str,
) -> Result<(&'a DddIdentity, &'a ParsedIdentity)> {
    let node = identities
        .get(id)
        .copied()
        .with_context(|| format!("DDD relation {role} identity `{id}` is not declared"))?;
    let parsed = parsed
        .get(id)
        .with_context(|| format!("DDD relation {role} identity `{id}` was not parsed"))?;
    Ok((node, parsed))
}

fn validate_relation_shape(
    kind: DddRelationKind,
    from_id: &str,
    from: &ParsedIdentity,
    to_id: &str,
    to: &ParsedIdentity,
) -> Result<()> {
    match kind {
        DddRelationKind::Contains => {
            if from.kind != DddIdentityKind::Context
                || to.kind == DddIdentityKind::Context
                || from.context != to.context
            {
                bail!(
                    "DDD relation contains must be context:<name> -> a member of the same context, got `{from_id}` -> `{to_id}`"
                );
            }
        }
        DddRelationKind::Uses => {
            if from.kind != DddIdentityKind::UseCase
                || !matches!(to.kind, DddIdentityKind::Aggregate | DddIdentityKind::Port)
            {
                bail!(
                    "DDD relation uses must be use-case -> aggregate or port, got `{from_id}` -> `{to_id}`"
                );
            }
        }
        DddRelationKind::DependsOn => {
            if from.kind != DddIdentityKind::Context
                || to.kind != DddIdentityKind::Context
                || from.context == to.context
            {
                bail!(
                    "DDD relation depends-on must be between two distinct bounded contexts, got `{from_id}` -> `{to_id}`"
                );
            }
        }
        DddRelationKind::Implements => {
            if from.kind != DddIdentityKind::Adapter || to.kind != DddIdentityKind::Port {
                bail!(
                    "DDD relation implements must be adapter -> port, got `{from_id}` -> `{to_id}`"
                );
            }
        }
        DddRelationKind::Realizes => {
            if from.kind != DddIdentityKind::Artifact
                || !matches!(
                    to.kind,
                    DddIdentityKind::Aggregate
                        | DddIdentityKind::UseCase
                        | DddIdentityKind::Port
                        | DddIdentityKind::Adapter
                )
            {
                bail!(
                    "DDD relation realizes must be artifact -> aggregate, use-case, port, or adapter, got `{from_id}` -> `{to_id}`"
                );
            }
        }
    }
    Ok(())
}
// HANDWRITE-END
