# Report review vocabulary

Distinctions a report must hold for the controller to accept it, accumulated
from rounds over ownership-, registry-, and publication-shaped code. They are
review heuristics, not dispatcher mechanics: read them while adjudicating a
`review` diff or report, and put the ones a given round actually turns on into
that round oracle `## Fabrication tells`, where they are frozen and checkable.

- Enforce frozen semantic distinctions in every reported row. A summary-level
  disclaimer does not cure a row-local contradiction (for example, declaring a
  key opaque globally but later treating its reuse as necessarily pointer or
  address reuse). Reject the report or record an explicit controller
  normalization before acceptance; never silently let the summary override the
  detailed evidence.
- Require one unambiguous target semantic owner per reported row. Current
  storage, target ownership, and service dependencies are separate fields; an
  `A / B` owner label or a process service named as a co-owner is not an
  ownership decision. Labels such as `candidate A or B`, `remove or retain`,
  and `optional compatibility owner` are unresolved choices and must be
  rejected or resolved by the controller before acceptance.
- Preserve sole-owner decisions from frozen design inputs. A new slice may
  add a typed binding or coordinator edge to an accepted aggregate, but must
  not silently nest that aggregate's records under a second owner or replace
  its key/value and ownership contract. Treat such a topology rewrite as a
  conflict requiring controller resolution, not harmless diagram shorthand.
- Keep current behavior separate from proposed target behavior. A stored
  configuration value is not evidence that the execution path consumes it,
  and a function name such as `*_all_threads` is not evidence of propagation.
  Verify the producer-to-consumer path before accepting the claimed effect.
- Do not project current broad invalidation onto a target authoritative
  version boundary. A current helper may clear caches after not-found,
  invalid, or no-op calls; a target publication generation advances only for a
  proved semantic visibility change. Preserve current reachability and target
  commit semantics as separate decisions.
- Keep a source-representable partial state or race window separate from an
  observed event. Separate calls without an atomic rollback protocol prove
  that skewed state is representable; they do not prove an exception, crash,
  callback, or failed publication actually occurs between those calls. Name
  the concrete event witness or report only the representable hazard.
- When the admitted state is read only through a helper, trace the helper's
  production callers before declaring its semantic consumers complete. Name
  each caller's actual enclosing function and branch; a helper labeled
  “general inquiry” can hide multiple behavior policies outside the selector
  file.
- For same-key or same-name replacement claims, distinguish paths that invoke
  the admitted writer from paths that do not. A second writer call may replace
  an entry, while an ordinary rebinding that skips that writer can leave the
  old entry stale. Do not summarize both as generic “rebind overwrites.”
- Keep entity identity separate from immutable publication version. Updating
  or decorating an existing entity normally preserves its typed runtime key
  while publishing a new definition/configuration version; only creation of a
  genuinely new entity allocates a new key. Require the report to name both
  transitions explicitly when same-display-name replacement is also in scope.
- If admitted evidence exposes a semantically duplicate owner outside the
  frozen selector, do not silently expand the denominator or pretend the
  duplicate was audited. Name its exact identity as an out-of-scope sibling
  inventory dependency, preserve the current denominator, and block a target
  “single owner” implementation until that sibling decision is accepted.
- Separate authoritative state failure from derived-projection maintenance.
  A publication/version update that defines visibility must fail closed, while
  an opportunistic cache insert, prune, or old-generation clear may safely
  skip only when a typed authoritative generation makes the projection
  correctness-neutral and lookup falls back to the owner.
- Helper names such as `set_field`, `replace`, `remove`, `clear`, or `drop`
  do not prove retain/release behavior. Inspect the helper body and record the
  exact old-value return/drop and new-value retain/transfer edges before
  calling an ownership ledger balanced.
- An ownership ledger for a registry is not complete until it also accounts
  for the fields of every object retained by that registry and for returned
  aliases. Require one row per stored field/alias with its incoming ownership
  contract, explicit retains, replacements, removed-value handling, and final
  retirement. Container-level `retain(entry)` evidence cannot authenticate
  `_target`, callback, payload, or child-handle ownership inside the entry.
- Distinguish an aggregate-owned cache/registry claim from a caller-owned
  alias or lease. A registry record such as `OwnedValue` is not itself a
  caller lease; report the registry's installed claim, each returned retain or
  `Arc` clone, and their independent retirement edges with the actual types.
- Do not invent a nested lease for an immutable value already owned by a
  leased aggregate. If `Arc<Aggregate>` keeps all aggregate fields alive, a
  proposed `Arc<Field>`, field-specific generation, or second retirement
  protocol needs a distinct independently-lived use case in the frozen
  contract. Otherwise require the field to remain a direct aggregate value and
  the aggregate lease to be the sole lifetime authority.
- A copied raw value, function parameter, vector lookup, or map insertion does
  not prove an incoming ownership transfer. Require the caller/container
  retirement edge too. If that edge is outside the allowed evidence surface,
  mark it unresolved rather than guessing `borrowed` or `transferred`.
- When one public constructor has wrapper, alias, or compatibility branches,
  audit each concrete variant. Do not project fields or registration behavior
  from one branch onto another; a branch that returns the input value may have
  no registry entry at all and a different retain ledger.
- Verify every claimed cleanup bypass against actual control flow. A
  side-channel exception flag is not a Rust early return or unwind, and a
  timeout path is not stale-state evidence when it still passes through the
  common cleanup block. Name the exact branch, return, panic, cancellation, or
  missing retirement edge that bypasses cleanup.
- Do not infer Rust guard lifetime from indentation or the apparent end of a
  method call. `if let`/`match` scrutinee temporaries can keep a `RefMut`,
  mutex, or container guard live through the branch body. Inspect the exact
  statement and workspace Rust edition; when release/reentry safety depends on
  it, use a same-shaped minimal probe or MIR evidence. Also distinguish the
  caller's registry guard from locks acquired inside helpers such as dict
  insertion: “no outer borrow” does not prove “release is guard-free.”
- Separate cleanup registration from cleanup effect. A central reset may call
  a named cleanup function whose body intentionally leaves state unchanged;
  that is an invoked no-op, not an absent call. Verify both the caller edge and
  the callee body before reporting either omission or successful cleanup.
- Include implicit storage retirement in lifecycle claims. Thread-local maps
  may survive object drop and runtime reset yet still be dropped by the OS
  thread's TLS destructor. Report each boundary separately; do not call such
  entries process-permanent, and check whether longer-lived objects lose their
  metadata when the creating worker exits.
- For callback/reentrancy claims, name the exact invocation site and the exact
  guards still live at that point. Distinguish a registry borrow/lock from an
  object-deallocation phase, collector phase flag, or graph traversal. Do not
  say all callbacks run on a notification path when that path only marks some
  entry kinds and invokes others.
- Keep stale map entries separate from leaked ownership after removal. Prove
  whether the key was removed on the concrete path before claiming address
  reuse; a missed cross-worker notification may leave a stale key, while a
  successful removal that skips `release` leaks an owned object without
  leaving a reusable map entry.
- Keep outer-container retirement separate from retained-object field
  retirement. Draining a registry normally retires its registry claim only;
  manually releasing all fields and then releasing a still-live entry can
  double-retire its contents. Require atomic replace-with-owned-return evidence
  for early field clearing and a separate account of external entry claims.
- Cleanup and reset evidence must include conditional acquisition and failure
  branches. `try_lock`, `try_borrow`, ignored `Result`, poison recovery, or an
  early return can make a cleanup silently partial or a no-op even when the
  nominal body clears every field.
- For concurrency claims, inspect synchronization inside every traversed
  helper and variant. Per-object read locks do not prove a graph-wide stable
  snapshot, but their presence also means the path must not be reported as
  wholly unlocked.
- A synchronization primitive's type name proves only its documented semantic
  contract, not a performance property. For example, `OnceLock::get_or_init`
  supports race-safe single publication; it does not by itself prove that
  concurrent initialization is lock-free. Separate initialization behavior
  from any post-initialization fast path and require evidence for both.
- A disabled or early-return branch can be a semantic no-op without being
  zero-overhead. Trace entry, first-use initialization, steady-state reads,
  allocation, and synchronization separately. Reserve "zero overhead" for a
  compiled-out or otherwise proven absent path.
- Do not equate debug-only with test-only. A `cfg(debug_assertions)` symbol can
  be production diagnostic state and can be consumed by integration harnesses
  or debug binaries. Classify a state identity as test-only only from its
  compilation/reachability boundary, not from a test-oriented name or caller.
- State synchronization internals only when the public contract or inspected
  implementation proves them. A public API may say concurrent callers wait or
  block without proving a mutex, futex, parking implementation, fairness, or
  another internal mechanism.
- When a report promises an exact implementation shape, require syntactically
  complete code: statics need initializers, types and visibility must be
  coherent, and the snippet must preserve the stated initial state. Conceptual
  pseudocode must be labeled as such and cannot satisfy an exact-path contract.
- Keep `changed paths` separate from `planned implementation paths`.
  `changed paths: none` is correct for a measure-only run but does not satisfy
  a ticket asking which files a later implementation must change. Require both
  fields when both are in the contract.
- Before accepting the final report, reconcile every acceptance verb that says
  print, list, enumerate, or provide a matrix with an actual report section and
  its complete members. A checklist `PASS`, `fully mapped`, or `unfinished:
  none` cannot substitute for omitted paths, rows, invariants, or test seams.
- Do not introduce a second selector denominator such as "matching lines",
  "unique lines", or "occurrence rows" unless the ticket defines it and the
  report recomputes it. If every appendix member has a distinct path:line, its
  physical-row count and matching-line count must agree.
- A complete state/operation matrix includes non-mutating error branches,
  read-only operations, RAII/destructor transitions, and reset/recovery paths,
  not only successful mutations. Likewise, `exact invariants` means an
  enumerated invariant set; two summary bullets cannot satisfy a larger frozen
  lifecycle/ownership contract.
- When target ownership moves state out of TLS, a process static, or a side
  table, recompute target retirement from the new owner. Do not copy a current
  storage defect such as TLS evidence loss at OS-thread exit into the target
  invariant; prove how the owning child/context record survives, publishes,
  joins, quiesces, and retires.
- Distinguish scoped child bindings from transferable child state. An active
  invocation/frame binding guarded by RAII normally belongs to one execution
  child and must be restored on guard drop; it is not automatically payload for
  thread snapshot/replace or worker inheritance. Likewise, compatibility
  cleanup must not clear a live scoped binding behind its guard. Require an
  explicit quiescence/fail-closed rule before accepting either transfer or
  cleanup of active child bindings.
- Separate a low-level helper's transition from its RAII wrapper's later drop.
  If an imbalanced helper returns `Err` while state remains active and the
  wrapper's destructor then marks it incomplete, report both steps. Do not
  attribute the destructor transition directly to the helper.
- When a ticket audits multiple reset, cleanup, or retirement entry points,
  require a row-by-path matrix over the whole admitted denominator. A prose
  description of the primary registry does not prove what each sibling cell,
  cache, handle, or retained value resets or leaves behind.
- Do not accept a number without a runnable selector, witnesses, and independent recomputation. Audit both admitted and discarded members.
- For an exact-set row table, independently compare every reported identity
  and its path/line/type fields with current selector output. A correct total
  or digest can coexist with fabricated, shifted, or reconstructed row
  evidence; reject those rows even when the arithmetic reconciles.
- For dense call-site matrices, resolve every row to its actual enclosing
  function boundary before naming the operation; do not infer owners by call
  order or copy a nearby sequence. Also prove guard lifetime at the exact call:
  a temporary `lock.write().method();` guard is normally dropped at the end of
  that statement, while a named guard remains live until its last use/scope.
  Reject a row that reports either form from visual proximity alone.
- Apply enclosing-function resolution to test rows too. A nearby test topic,
  filename, or later assertion does not authenticate the `#[test] fn` that
  owns a selector line; report the actual enclosing test name.
- A mutation/cleanup helper call does not prove that the surrounding operation
  changed state or succeeded. For every matrix row, inspect whether the call is
  outside a `match`/error branch, runs on empty/no-op inputs, or still runs
  after an exception is raised. Preserve these call-site reachability facts
  separately from the desired target semantics.
- Apply the same current-selector check to every exhaustive evidence inventory,
  not only the primary exact-set table. Caller/acquisition families can be
  omitted while the admitted digest stays correct, and path:line witnesses can
  be stale, snippet-relative, or reconstructed. Compare the complete reported
  evidence surface to fresh selector output and reject any missing family or
  mismatched path/line/type claim.
- For selector family breakdowns, every counted row must actually match that
  selector. Adjacent declarations or mutation statements may be useful context
  but must be explicitly excluded from the selector count; never let a correct
  headline total authenticate a fabricated family subtotal.
- A selector family subtotal used for acceptance must expose its complete
  member path:line list in the report or a protected machine-readable sidecar.
  Representative witnesses can illustrate a family but cannot authenticate
  the subtotal or prove that every headline member was assigned exactly once.
- Reconcile every declared numeric count with the enumerated surface in the
  same report. Section headings, terminal summaries, and acceptance checklists
  must update when a revision splits or appends rows; a correct list with a
  stale headline count is not self-reconciling evidence.
- For an alternation selector, the default denominator is the selector's
  physical output rows, not one row per matched alternative. A source line
  mentioning multiple identities appears once and records all matched
  identities (for example `symbols=[A,B]`). If the ticket instead requires a
  symbol-occurrence denominator, it must say so explicitly and the controller
  must independently recompute that different denominator.
- State whether an exact selector is comment-inclusive or code-reference-only.
  When the ticket asks for actual state accesses, freeze a selector that
  excludes comments by construction and count only declaration/read/write
  code rows. Do not count a test-name comment as a test access while excluding
  an equivalent production comment. If comments are intentionally admitted,
  list them as a separate comment-only category and reconcile them explicitly.
- Validate every cited representative witness too. Correct aggregate counts do
  not authenticate sample rows: each cited path/line/type and the semantic fact
  attributed to it must match fresh current source. Reject stale or invented
  samples even when the selector total and headline conclusion are correct.
- Bind every exact-set row to its actual enclosing function/module from fresh
  source, preferably with a mechanically derived row-to-owner map when the
  language shape permits it. Do not infer the owner from a stale line number,
  nearby comment, expected behavior, or previous revision. A correct row count
  with wrong enclosing owners or policies is not reconciled.
- Reject placeholders such as “remaining rows cataloged”, “same schema”, or
  ellipses in an exact-set report. If complete witnesses do not fit, keep the
  roll-up open and create smaller owner slices.
- Keep the rendered report compact enough for the dispatcher to normalize.
  When a raw log contains `## EXEC REPORT` but `status` is `EMPTY` and no
  normalized report artifact exists, verify the snapshot, then resume
  the same conversation with an explicit character budget and the exact
  evidence surfaces that must remain. Never accept the raw log as the report.
- A PASS checklist or sentence saying an audit was completed is not the
  required audit surface. When the ticket requires a path matrix, ownership
  ledger, caller lineage, or retain/release accounting, the report must print
  those rows with evidence; reject a summary that merely asserts completion.
- Apply that rule to implementation-boundary deliverables too. If acceptance
  requires the smallest safe slice, exact source paths, invariants, forbidden
  changes, or focused tests, the report must print each of those surfaces.
  Naming a target struct and then marking the criterion PASS is not a
  substitute for the requested path/test matrix.
- Verify every claimed existing path and fixture leaf against the fresh
  workspace. A semantically plausible but nonexistent filename is a failed
  witness. Keep planned new paths explicitly labeled as planned so they are
  not subjected to the existing-path check.
- Treat numeric test/invariant minimums as floors, never as permission to
  truncate an enumerated coverage contract. Map every explicitly required
  seam to a distinct planned test or state exactly which test covers multiple
  seams and how. If the oracle lists more seams than its numeric minimum, the
  full seam list controls acceptance.
- A future verification gate is not an executed result. If the profile forbids
  tests/builds, the report may name an exact test or command as pending
  controller verification, but it must not say that test passes.
- Preserve the oracle's epistemic boundary. A source comment, issue history,
  design rationale, or suspected mechanism can explain why code exists, but it
  does not prove the mechanism, minimal critical section, safety property, or
  recovery policy. If the ticket reserves that proof for later work, reject a
  report that upgrades the motivation into a present fact.
- Failure evidence must cover every owner that can be partially advanced, not
  only the headline cache or registry. An insertion that occurs after a
  fallible operation can prove the map stayed clean while the retained module,
  allocator, service, or external resource became unsafe to reuse. Require the
  retry, abandonment, poison, or fail-closed decision for that retained owner;
  absence of a cache entry is not integrity evidence.
- Keep the failing synchronization attempt separate from later poison
  handling. A panic while a mutex guard is live drops that guard during unwind
  and poisons the mutex; `PoisonError::into_inner()` can describe a subsequent
  acquisition policy, not recovery of the already failed operation.
- An `Err` from a mutating API proves neither atomic rollback nor partial
  mutation unless its contract or implementation says so. Report the bounded
  facts (which later publications did not occur, which owner remains live, and
  what atomicity is unproven) instead of asserting that a declaration,
  allocation, or registry mutation definitely survived the error.
- Reconcile symbol-occurrence subtotals at row granularity. When one physical
  row contains two different alternatives, list both once; do not attribute a
  duplicate occurrence to the next line or say one token appears twice unless
  the source line actually contains it twice.
- On revision, propagate a corrected fact into every report surface that
  repeats it. A fixed criterion summary does not cure a stale appendix,
  occurrence label, matrix cell, invariant, or conclusion that still states
  the rejected version. Cross-check the replacement report internally before
  acceptance.
- Keep explicit retirement, language/runtime destructors, and OS process
  reclamation separate. A process-global static is not proven to run a Rust
  destructor at process exit; when there is no explicit drain, state that the
  address space is reclaimed without claiming Python retains or fields were
  retired through their normal ownership protocol.
- Avoid using “lock-free” to mean only “outside another lock.” Say
  `registry-guard-free`, `outer-lock-free`, or name the exact absent guard while
  still listing any object/inner lock that remains required.
- Keep owner visibility separate from leased-object lifetime. Removing a
  registry entry, handle, or lookup edge can make new resolution fail
  immediately while an already-acquired `Arc`, refcount, epoch pin, or other
  lease keeps the detached object alive. Reject reports that delay
  deregistration until lease count reaches zero, or that claim deregistration
  invalidates existing leases, unless the implementation proves that coupling.
- A raw callback, vtable slot, JIT address, or FFI function pointer proves
  callable identity, not that its code remains mapped. Require explicit
  code-lifetime authority: a proved process-image lifetime or an owned module
  lease. An optional authority field does not close the lifetime invariant when
  unloadable producers are in scope.
- Do not call a multi-step fallible publication “atomic” merely because its
  target invariant is all-or-nothing. Require the concrete commit point,
  provisional visibility rules, and rollback for every acquired claim and
  partially inserted entry. Use `transactional publication` unless one actual
  atomic primitive spans the entire visible transition.
- Reconcile the visibility set with the phase ordering. Every alias, binding,
  record, policy, and cache generation claimed to become visible together must
  be staged before and activated by the named commit point. A supposedly
  coordinated alias or index update cannot first happen after that commit;
  post-commit work is limited to prior-generation detachment and retirement.
- Do not infer ancestry or propagation from local variable names such as
  `parent`, `snapshot`, or `context_snapshot`. Trace where the value was
  captured and on which OS/logical thread before claiming inheritance.

