<!-- HANDWRITE-BEGIN gap="missing-generator:unit-test:0882defa" tracker="pending-tracker" reason="Minimal WI fixture with the canonical bounded sections and artifact references." -->
# Fixture Work Item

## Problem

Typed AW artifacts need navigable, read-only context.

## Capability Alignment

Capability: Optional AW Typed Context
Capability Gap: Typed structure is not visible in generic Markdown.

## Scope

Render one confined artifact without lifecycle mutation.

## Acceptance Criteria

- The source remains byte-identical after render and refresh.
- The related `tech-design.md` remains canonical repository content.

## Reference Context

Parent Epic: #2196
Related EC: `external-contract.md`
<!-- HANDWRITE-END -->
