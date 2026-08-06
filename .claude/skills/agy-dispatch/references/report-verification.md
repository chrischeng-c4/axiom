# Verifying an AGY report

Reject a report if it has no reproducible selector, no witnesses for its headline number, a missing oracle hard-floor member, an unexpected repository mutation, or a changed protected artifact hash.

Reject an empty local report or one that does not begin `## EXEC REPORT`.
GitHub comments are unverified side effects; the captured local report is the
dispatch result. Confirm the stored rendered-prompt digest and oracle digest
before evaluating the report.

`status: EMPTY` remains a failed delivery even if the raw run log visibly
contains a report. After snapshot and permission-state verification, resume a
ticketed conversation with a compact-output budget. For one-shot work, record
the failed run and retry under a new run id instead. Require the normalized
`*.report.md` artifact before semantic acceptance.

For a derived set, independently enumerate the full candidate surface before looking at AGY's shortlist. Recompute both admissions and exclusions from the candidate surface. A selector cannot demonstrate an omitted suite; the oracle hard floor can.

For every accepted number, record the independent command or method, fields recomputed, known impurities, and the verdict. Close a ticket only after posting that verdict; record a one-shot verdict in the controller log.

For denied runs, inspect the log for the command that was refused. If it is not
needed, tighten the prompt and resume the existing ticketed conversation, or
retry one-shot work under a new run id. If it is a reusable project capability,
add the narrow Project-scope rule once, update the profile's exact
`project_permissions`, rerun `doctor`, and take a fresh snapshot. Resume only
ticketed work. Never mutate permissions only for one task and never use a
bypass-permissions flag.

For revision waves, compare the resulting diff with the injected delta
contract. Use `path_change_budgets` to catch formatting explosions, but do not
mistake a line budget for semantic verification.

Compare a revised report with every previously accepted section. The latest
normalized report must be independently complete; it cannot rely on a prior
report for rows, matrices, lifecycle boundaries, invariants, paths, forbidden
changes, or tests. Reject revision compression that fixes one finding by
dropping accepted evidence elsewhere.

Treat PASS as provisional even when the report is detailed. Independently run
`git diff --check` for cleanliness claims. For every claimed unchanged or
out-of-scope form, require a focused negative control or inspect the production
eligibility branch directly; repeating the task's exclusion list in the
report is not evidence.

For ownership inventories, reject ambiguous target-owner labels. Distinguish
the observed storage location, exactly one semantic owner in the target
design, and any process or platform service that owner calls. Also trace a
claimed behavior through its consumer: API names, comments, and writable
configuration slots do not prove inheritance, propagation, or runtime effect.
An ownership cell must contain one decision, not `candidate`, `A or B`,
`remove or retain`, or another unresolved alternative.

For cleanup-bypass claims, trace control flow through the cleanup statement.
Language-level side-channel exceptions do not imply host-language early
return, and timeout/cancellation only creates stale state if that concrete path
actually skips or omits retirement.

Treat checklist PASS labels as claims, not evidence. Required matrices and
ownership ledgers must be present row-by-row. For propagation claims, trace the
value's capture site and execution thread; a variable named `snapshot` does
not prove that it came from a parent or caller.

For ownership ledgers, inspect mutator helper bodies. A `set`, `insert`,
`replace`, `remove`, or `drop` name does not establish that the old value was
released or the new value retained.

Trace ownership through nested retained objects, not only the outer registry.
List each stored field and returned alias, its incoming ownership convention,
extra retains, replacement result, and retirement. If the caller-to-callee
ownership contract is not proven, mark that edge unresolved and request the
smallest source witness; do not silently classify a raw copied value as
borrowed or transferred.

An `MbValue`/handle parameter copied from a vector and inserted into a map is
not proof of transfer. Verify what later retires the argument container and
whether that path releases the same slot. If it is outside the ticket's
evidence surface, keep the edge unresolved and design a typed ownership seam.

Audit every constructor variant separately when wrapper/alias/compatibility
branches create different fields, retains, or registry entries. For callbacks,
verify the exact function that invokes user code and list the actual live
guards at that point. A registry removal that has already released its borrow
is not a lock-held callback, though execution may still be unsafe because it
occurs inside object deallocation or a collector phase.

Separate two failure classes in retirement reports: a stale key that remains
address-reusable, and an owned retain leaked after its map entry was removed.
They need different evidence and different target remediation.

When proposing retirement, do not conflate releasing the registry's retain
with destructing fields of the retained object. Early field clearing requires
an atomic replacement that returns the old owned value; final object
destruction remains responsible for fields still installed.

For cleanup paths, include failed `try_*` acquisition and ignored-result
branches. For concurrency paths, distinguish per-object locking from
aggregate/graph consistency; neither collapse "some locks" into safe nor
"no global coordinator" into wholly unlocked.

For central cleanup, verify two independent facts: whether the coordinator
calls the cleanup entry point, and what the callee actually clears. An invoked
empty/no-op cleanup is not a missing call and is not effective retirement.
Also account for implicit owner destruction such as TLS teardown at OS-thread
exit. Object retirement, runtime reset, worker exit, context retirement, and
process exit are distinct lifecycle rows.

For cleanup inventories, cross every admitted identity with every in-scope
cleanup, reset, and retirement path. Record direct reset, indirect effect, or
unchanged for each cell. Verify that pointer-cache invalidation and owned-value
release happen before the owner can be removed; clearing the headline map does
not authenticate sibling-state cleanup.

For selector-family subtotals, require complete membership evidence: each
selector match must appear exactly once under a family in the report or a
protected generated sidecar. Function declarations, nearby comments, and a few
representatives are context, not members of a path:line selector denominator.
For alternation selectors, count physical selector output rows by default.
List every identity matched on a multi-identity line in that one row rather
than duplicating the path:line under several families. Use a symbol-occurrence
denominator only when the ticket explicitly defines and freezes it.

For a dense producer/caller matrix, a correct path:line selector is not enough:
map each row to the nearest enclosing function definition and inspect that
function's control flow. Never advance operation names mechanically from one
row to the next. For lock-state columns, distinguish named guards that remain
in scope from temporary method-chain guards that are dropped at statement end;
nearby lock acquisition is not evidence that a guard is live at the call.
Likewise, the presence of a marker/cleanup call is not evidence that the
operation succeeded or changed data: prove whether empty inputs, no-op
branches, and raised-error paths still reach the call.

When an acceptance criterion asks for an implementation slice or test strategy,
verify that the report actually prints exact changed paths, invariants,
forbidden changes, and concrete focused test seams. A PASS checklist cannot
authenticate an omitted delivery surface.

Do not infer lock freedom, wait freedom, or contention cost from primitive
names such as `OnceLock`, atomics, or `parking_lot`; prove the concrete phase
and path. Also keep planned gates distinct from executed evidence: a
measure-only run that was forbidden to test may name a test, but cannot report
that it passed.

Do not infer zero runtime overhead from "disabled", "disarmed", or "no-op".
Audit first-use initialization and steady-state entry/read costs; only a
compiled-out or otherwise absent path supports a zero-overhead claim. Likewise,
`cfg(debug_assertions)` is debug-only, not necessarily test-only. Require a
`cfg(test)` or equivalent reachability proof before classifying the identity as
test-only.

Keep public synchronization guarantees separate from internal implementation.
If the API says concurrent callers can block, report that guarantee without
inventing a mutex, futex, parking strategy, fairness property, or cost model.
For an exact proposed code shape, verify that the snippet is syntactically
complete and preserves the required initializer; otherwise treat it as
conceptual pseudocode rather than implementation evidence.

Distinguish repository mutations from future delivery design. A measure-only
report can correctly say `changed paths: none` and still fail because it omits
the separately required planned implementation paths. Audit every acceptance
verb (`print`, `list`, `enumerate`, `matrix`) against the actual payload; a
criterion marked PASS or a statement such as `fully mapped`/`unfinished: none`
is not evidence when the members themselves are absent.

Reject an unexplained second denominator. If the selector prints 39 distinct
path:line rows, a claim that they span 37 matching lines is contradictory even
when the appendix itself is complete. For state machines, verify every
operation against every relevant state, including unchanged error/self-loop
cells, read-only snapshots, guard/destructor behavior, and test-only recovery.
Require the exact invariant list itself, not a short "invariants" heading.

Re-derive lifecycle after an ownership migration. Current TLS teardown may
discard evidence, while target child-owned state can remain in a domain record
until join/quiescence; copying the old teardown behavior into target invariants
is a semantic error. Also split helper return behavior from RAII cleanup: an
`Err` that leaves state active is not itself the later guard-drop transition to
incomplete.

Treat comments, issue narratives, and design motivation as hypotheses unless
the report supplies the proof required by the oracle. They cannot establish a
specific syscall, process-global hazard, minimal lock boundary, safety
property, or recovery policy merely because the implementation cites them.
Preserve explicit "unproven", "suspected", and future-ticket boundaries.

On a failure path, enumerate every object already mutated before the error.
A clean cache/map can coexist with a partially defined module, consumed input,
allocated external resource, or poisoned service. Verify the retained owner's
next-call behavior and the reviewed retry/abandon/poison/fail-closed policy;
do not accept "not inserted" as proof that retry is safe.

For poisoned locks, write a two-attempt timeline: the failing attempt unwinds,
drops its live guard, and poisons the mutex; only a later acquisition observes
`PoisonError` and may call `into_inner()`. Do not call that later policy
recovery of the failed operation.

For a fallible mutator, verify documented/implemented error atomicity before
saying its mutation survived or rolled back. If that proof is outside the
surface, state only what is observable after the call and mark mutation state
unresolved.

For symbol-occurrence counts, compare each row's actual token multiplicity.
Two alternatives on one line are two distinct occurrences; the next line does
not inherit a duplicate. Require the appendix wording to match the same
row-local arithmetic as the subtotal.

After any revision, search the replacement report for the rejected claim and
all equivalent labels. The summary, appendix, matrices, invariants, and tests
must agree; a corrected headline with stale detailed rows is still a failed
delivery.

Do not infer static destruction at process exit. Record explicit drains and
normal owner destructors only when a reachable call/path proves them; otherwise
state that OS address-space reclamation ends remaining process state without
normal Python ownership retirement.

“No registry guard held” is not “lock-free” when an object-field or inner lock
still protects the mutation. Name the exact absent and live guards.

When a registry stores a leased object, verify two timelines independently:
lookup visibility and object destruction. Entry removal may reject all new
lookups while pre-existing `Arc`/refcount/epoch leases continue safely. Do not
accept “remove when active count reaches zero” or “removal invalidates active
calls” without a proven design that intentionally couples those events.

For native/JIT callback catalogs, verify code lifetime separately from record
and call lifetime. Copying a function pointer, raw address, or vtable entry
does not keep an unloadable module or generated-code allocation mapped. Require
either proved process-image lifetime or an owned module/code lease in every
published record; optional lifetime authority leaves a dangling-call gap.

Distinguish atomicity from transactional rollback. A sequence that reserves an
ID, acquires owned claims, constructs an object, inserts an entry, and installs
a handle is not one atomic operation by assertion. Require an exact commit
point, proof that provisional state is not externally visible, and rollback at
each failure boundary; otherwise record the atomicity as unproven.
