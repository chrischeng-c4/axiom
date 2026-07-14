# #1627 — `__enter__` returning a non-self value must retain it

Status: landed `b789a7900` (2026-07-14). Backfill TD.

## Mechanism

The with-statement exit lowering performs a double-release on the context
value (explicit release + `Copy`'s auto release-before-overwrite, #1129 R2);
every `mb_context_enter` branch compensates with a retain. `TemporaryDirectory.__enter__`
returns `name` (not `self`) and retained only `name`'s own creation — the
missing compensating `retain_if_ptr(recv)` left a use-after-free that
SIGTRAP'd intermittently (`_xzm_xzone_malloc_freelist_outlined`), masked for
weeks because an ingress wall compile-rejected the victim fixtures until
#1595 removed it.

## Invariant

Every `__enter__` implementation (native or synthesized) must leave the
with-protocol's refcount contract satisfied for BOTH the receiver and the
returned value — audit any `__enter__` that returns something other than
`self` for the compensating retain.

## Verification contract

`behavior/std-libs/tempfile/temporary_directory_cleanup_on_exit.py`,
`real_world/std-libs/errno/translate_oserror_errno_to_name.py` — 5/5 repeated
runs identical (intermittency gone); MallocScribble-clean in debug.
Attribution lesson recorded: intermittent crashes defeat single-sample
bisects — A/B with repeated sampling before blaming a commit.
