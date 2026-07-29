# Process string-hash state topology

Issue: #3003
Parent inventory: #2968
Source revision: `7897037c40`

This Stage 1 slice classifies the randomized state shared by ordinary-string
and surrogate-codepoint hashing. The current storage already has the required
process-immutable ownership, so this design records its invariants and gates
without proposing a source migration.

## Bounded context

```text
Process
└── StringHashState
    └── RandomState

ExecutionContext
└── ObjectDomain
    ├── string objects
    ├── dictionaries with string-like keys
    └── sets with string-like elements
```

`StringHashState` is process immutable. Execution contexts, logical children,
and OS workers consume it but never own, reset, or replace it.

## Aggregate and values

| Type | Kind | Identity / value |
|---|---|---|
| `StringHashState` | process-immutable service value | one published process state |
| `RandomState` | immutable hash-builder state | randomized process keys |
| `StringHash` | value | runtime `i64` projection |
| `CodepointHash` | value | hash of surrogate-preserving `u32` sequence |

The current `OnceLock<RandomState>` is the accepted implementation. Its
semantic contract is race-safe single publication. This design does not claim
that concurrent initialization itself is lock-free. After initialization,
consumers borrow the one published immutable value.

## Frozen inventory

The one production identity has sorted newline-terminated SHA-256
`bbd3410cb57f52a8cce1ee47933353657cccda01dfcead6f71512f7e80cb4031`.
There are zero test-only state identities.

| Current symbol | Storage | Role | Target disposition |
|---|---|---|---|
| `STRING_HASH_STATE` | process `OnceLock<RandomState>` | shared randomized builder | retain as process-immutable `StringHashState` |

The accepted selector contains 13 physical rows:

| Family | Rows |
|---|---:|
| `STRING_HASH_STATE` | 2 |
| `string_hash_state` | 3 |
| `string_hash_value` | 5 |
| `codepoints_hash_value` | 3 |
| **Total** | **13** |

## Consumer matrix

| Path | Consumer | Behavior |
|---|---|---|
| `dict_ops.rs:938` | `dict_string_hash_value` | hashes ordinary string-like dict keys |
| `dict_ops.rs:942` | `dict_codepoints_hash_value` | hashes surrogate-codepoint dict keys |
| `string_ops.rs:2502` | `mb_str_hash` surrogate branch | hashes preserved codepoint sequence |
| `string_ops.rs:2505` | `mb_str_hash` ordinary branch | hashes ordinary nonempty string payload |

The two selector test rows at `string_ops.rs:6602-6603` hash the same generated
key twice and assert intra-process stability. They are test calls, not
additional state identities or production consumers.

## Initialization and hashing behavior

The first nonempty ordinary-string hash or any codepoint hash calls
`string_hash_state().get_or_init(RandomState::new)`. Concurrent callers observe
the `OnceLock` single-publication contract: one initialized value becomes the
process state and all successful callers receive it.

Ordinary empty strings return zero before touching the `OnceLock`. An empty
codepoint slice does not use that shortcut; it initializes/uses the process
builder and hashes the slice. The two paths must not be collapsed in tests or
documentation.

Each hash operation creates a fresh hasher from the same `RandomState`, feeds
the value, and projects `finish() >> 17` to the runtime integer domain.
Identical inputs within one process therefore use stable builder keys.

## Lifecycle invariants

`cleanup_all_runtime_state` does not reset `STRING_HASH_STATE`. That is
required, not missing cleanup.

Changing the builder while any live dict/set contains hashed string-like keys
would make future lookup choose buckets using different hash keys. A
per-context, per-worker, or per-run reset could therefore make present keys
unfindable or split equality/hash behavior across threads.

Required invariants:

1. At most one `RandomState` is published per process.
2. Publication is visible to all workers and contexts.
3. No consumer mutates or replaces the published state.
4. Runtime reset does not change hash keys.
5. Empty ordinary string remains the exact zero special case.
6. Ordinary and codepoint hashing use the same published builder.
7. Live container lookup never crosses a hash-state generation boundary.
8. Process exit is the only retirement event.

## `PYTHONHASHSEED` boundary

The scanned Mamba runtime contains no source path that parses or applies
`PYTHONHASHSEED` to this state. `RandomState::new` supplies randomized keys, but
that is not evidence of CPython-compatible seed selection or deterministic
seed control.

This ownership slice makes no `PYTHONHASHSEED` capability claim. Adding that
feature would require a separate process-bootstrap design, reproducibility
oracle, and proof that configuration is fixed before first hash access.

## Source implementation boundary

Source implementation paths: none.

Forbidden changes:

- converting the state to TLS or context-local storage;
- resetting it from runtime cleanup;
- mutable seed reconfiguration after publication;
- separately seeded ordinary/codepoint hash builders;
- removing the ordinary-empty-string zero behavior;
- claiming `PYTHONHASHSEED` support without a bootstrap implementation.

No `projects/mamba/src/**` implementation ticket is generated from #3003.

## Verification gates

- Exact-set gate: one identity and all 13 rows remain reconciled.
- Concurrent-initialization gate: barrier-controlled first access from multiple
  OS threads returns hashes derived from one published state.
- Worker-visibility gate: the same nonempty string hashes identically on every
  worker in one process.
- Context-visibility gate: two execution contexts share hash results without
  sharing mutable context state.
- Reset-stability gate: hash a key, retain a container holding it, run permitted
  runtime cleanup, and prove lookup/hash stability.
- Empty-path gate: ordinary empty string returns zero without forcing state;
  empty codepoints follow the codepoint builder path.
- Consumer-parity gate: dict and `mb_str_hash` paths agree for the same
  ordinary/codepoint payload.
- Negative seed gate: no Mamba source claim or behavior is attributed to
  `PYTHONHASHSEED`.
- Existing focused tests such as `test_hash_same_string_same_result` and
  `test_hash_common_prefix_flood_keys_remain_distinct` are named gates only;
  AGY's measure-only run did not execute them.

## Dependency and dispatcher result

- #3003 is a Stage 1 classification slice under #2968; it changes no source.
- #2968 must close before Stage 2 #2839 can be dispatched.
- AGY produced the exact identity, 13-row appendix, consumer matrix, and
  no-source boundary in its first normalized report.
- The controller rejected two unsupported result claims: `OnceLock` did not
  prove lock-free initialization, and a forbidden-to-run unit test could not be
  reported as passing. The corrected classification is accepted but is not a
  one-pass ramp sample.
