---
change: lens-full-upgrade
group: new-languages
date: 2026-03-13
---

# Requirements

Add 4 new languages: (a) TOML — serde-based checker for syntax + schema validation (Cargo.toml, pyproject.toml). (b) SQL — tree-sitter-sql grammar, injection detection in string literals of Python/JS/Go code, basic syntax validation. (c) Proto/gRPC — tree-sitter-protobuf grammar, proto3 syntax validation, field number uniqueness, service/method structure. (d) GraphQL — tree-sitter-graphql grammar, schema validation, query complexity hints, deprecated field usage.
