# Mesh

## Brief

Mesh is the relationship/property-graph service in the Axiom service stack.
It owns typed node/edge storage with properties, and traversal/path query
over that graph. Like `lumen`, mesh is log-driven and derived: writes fold
through a raft-replicated log into a separate, rebuildable local index. The
caller owns the system of record; mesh never becomes the durable owner of
relationship data.

It is intentionally separate from its siblings: `beam` owns vector ANN
search, `lumen` owns lexical/semantic/perceptual search and dedup, and
`cube` owns OLAP-style columnar aggregation. Mesh owns the graph shape:
nodes, typed edges, properties, and traversal.

## Contributing

Repo-wide authoring rules live in
[../../CONTRIBUTING.md](../../CONTRIBUTING.md).

## Capability Contract

Machine-readable capability contract for Mesh. Full contract:
[CAPABILITIES.md](CAPABILITIES.md).
