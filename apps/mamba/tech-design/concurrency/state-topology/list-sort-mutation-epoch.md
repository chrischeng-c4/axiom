# List sort mutation epoch state topology

Issue: #3001
Parent inventory: #2968
Source revision: `1d07ce5c6f`

This Stage 1 slice classifies the TLS watch used by `list.sort(key=...)` to
detect mutation during user callbacks. It defines an object-owned mutation
epoch and a child-owned sort frame that remain observable across worker
migration and concurrent mutation without changing `src/**`.

## Bounded contexts

```text
ExecutionContext
├── ObjectDomain
│   └── MbListState[ListObjectIdentity]
│       ├── buffer: MbRwLock<MbListBuffer>
│       └── mutation_epoch: AtomicU64
└── ExecutionChild
    └── ListSortOperationFrame
        ├── target: RetainedListHandle
        └── baseline: ListMutationEpoch
```

`ListSortOperationFrame` is the child-owned semantic owner of one active sort
observation. The mutation epoch belongs to the list object and dies with that
object. The frame observes the epoch; it does not own it.

TLS is not a compatibility owner for either value. Existing C-ABI entry points
reach `MbListState` through their explicit `MbValue` receiver.

## Aggregate and values

| Type | Kind | Identity / value |
|---|---|---|
| `MbListState` | object entity | typed list object identity and generation |
| `ListMutationEpoch` | monotonic value | non-wrapping `u64`, with `MAX` exhausted |
| `ListSortOperationFrame` | child-owned lexical aggregate | one active sort attempt |
| `RetainedListHandle` | owned RAII value | one balanced retain/release of the target |
| `MutationObservation` | value | unchanged, changed, or epoch-exhausted |

`MbListState` wraps the current `MbRwLock<MbListBuffer>` and forwards
`read`, `write`, and `try_write` so existing `ObjData::List(ref lock)` access
shape can migrate without process-wide registry lookup.

`RetainedListHandle` performs an explicit retain at frame construction and a
balanced release on `Drop`. The frame never stores a raw target pointer.

## Frozen inventory

The two production identities have sorted newline-terminated SHA-256
`0a78f648d35f51739e8969f2f786ac05071ff6783a6e8d62652b0f434c8b8c06`.
There are zero test-only state identities.

| Current symbol | Current storage | Current role | Target disposition |
|---|---|---|---|
| `SORT_MUTATION_WATCHES` | TLS `RefCell<Vec<SortMutationWatch>>` | active raw-list-address watches | remove; replace with child sort frame plus object epoch |
| `NEXT_SORT_MUTATION_GUARD_ID` | TLS `Cell<usize>` | worker-local watch token allocator | remove; frame identity is lexical |

The accepted selector contains 32 physical rows:

| Family | Rows |
|---|---:|
| `SORT_MUTATION_WATCHES` | 5 |
| `NEXT_SORT_MUTATION_GUARD_ID` | 2 |
| `ActiveSortMutationWatch` | 5 |
| `mark_list_mutated` | 20 |
| **Total** | **32** |

The 20 `mark_list_mutated` rows are one definition plus 19 production call
sites. `ActiveSortMutationWatch` has four production rows and one unit-test
construction.

## Current watch lifecycle

`ActiveSortMutationWatch::new` converts the list pointer to `usize`, allocates
a worker-local wrapping guard ID, and pushes `{guard_id, list_id, mutated:
false}` into the TLS vector.

Every `mark_list_mutated(list)` scans that worker's vector and sets all entries
whose raw `list_id` matches. `mb_list_sort_kwargs` reads the flag after every
key callback and again after sorting the key/value pairs. `Drop` removes the
first entry with the guard ID.

Nested watches for the same list work only when all operations remain on the
same OS worker: one mark updates every matching entry. Rust early return and
unwind run `Drop`, so a successfully constructed watch normally cleans up.
There is no central runtime reset for either TLS identity.

## Current producer matrix

“Mark reached” is intentionally separate from “content changed.” The first
migration preserves the exact legacy reachability of all 19 calls; later
CPython behavior refinement requires a separate ticket and oracle.

| Line | Enclosing operation | Guard live at mark | Mark reachability |
|---:|---|---|---|
| 273 | `mb_int_list_append_raw` | named `buf` write guard | valid list; one integer appended |
| 285 | `mb_float_list_append_raw` | named `buf` write guard | valid list; one float appended |
| 974 | `mb_list_setitem` | named `items` write guard | valid index; also marks an equal-value replacement |
| 1204 | `mb_list_setslice`, extended | named `items` write guard | equal cardinality; includes empty/empty or equal replacement |
| 1231 | `mb_list_setslice`, contiguous | named `items` write guard | step one; includes empty/empty or equal replacement |
| 1266 | `mb_list_delslice`, step one | named `items` write guard | valid nonzero step; includes an empty range |
| 1314 | `mb_list_delslice`, other step | named `items` write guard | nonzero nonunit step; includes zero selected positions |
| 1357 | `mb_list_delitem` | named `items` write guard | valid index only; out-of-range is a silent unmarked no-op |
| 1483 | `mb_list_append` | temporary guard already dropped | valid list; retained item appended |
| 1504 | `mb_list_append_unchecked` | temporary guard already dropped | valid list; `store_owned` still runs before append |
| 1532 | `mb_list_insert` | named `items` write guard | integer index; one item inserted |
| 1554 | `mb_list_pop` | named `items` write guard | nonempty list; empty error returns before mark |
| 1574 | `mb_list_pop_at` | named `items` write guard | valid index; range error returns before mark |
| 1615 | `mb_list_remove` | named `items` write guard | snapshot match still occupies the same slot |
| 1676 | `mb_list_extend`, known | temporary guard already dropped | recognized collection; marks even when the source is empty |
| 1691 | `mb_list_extend`, generic | loop guards already dropped | valid iterator; marks even when it yields zero values |
| 1704 | `mb_list_clear` | named `buf` write guard | valid list; marks an already-empty list |
| 1716 | `mb_list_reverse` | temporary guard already dropped | valid list; marks length zero or one |
| 2551 | `mb_list_repeat_inplace` | temporary guards already dropped | after every count branch, including count one and raised `TypeError` |

The last row is a current anomaly: `None` raises `TypeError` through Mamba's
pending-exception channel and then still reaches the marker. The migration
records the same epoch event so it does not silently alter the existing watch
contract. A conformance ticket may later move that event behind a successful
mutation decision.

## Current defects

### Worker-local visibility loses concurrent mutation

A mutation on worker B consults B's TLS vector. A sort frame installed on
worker A is therefore unchanged even though both operations target the same
list. Migrating the logical execution child between workers has the same
failure mode.

The target epoch is stored with the list, so all workers observe one mutation
sequence without a process-global lock.

### Raw address and wrapping token are not authority

The current watch key is an unretained raw address. If lifetime assumptions
change or a stale entry survives, allocator reuse can make a new list match an
old watch.

`NEXT_SORT_MUTATION_GUARD_ID` wraps and skips zero. After a complete `usize`
cycle, two simultaneously live watches can receive the same token; dropping
one may remove the other.

The target retained handle prevents list retirement while the frame lives.
Lexical frame identity eliminates the token allocator.

### Check and write-back are not atomic

The current final flag check occurs before acquiring the write lock used to
replace the list buffer. A concurrent mutation can occur after that check and
before write-back, then be overwritten by the sort.

Target write-back acquires the list write guard and rechecks the epoch while
the guard is held. It writes only when the epoch still equals the baseline.

### Temporary guards expose an epoch-publication gap

Several producers currently drop a temporary list write guard before calling
the marker. A sort could acquire the buffer between the content change and
epoch publication.

The source migration publishes the epoch inside the same list critical section
as each content change. Existing marker-only no-op/error events remain
published at their current reachability boundary.

## Target observation contract

```mermaid
sequenceDiagram
    participant Sort as ListSortOperationFrame
    participant State as MbListState
    participant Key as key callback
    participant Mut as concurrent mutator

    Sort->>State: read lock; retain snapshot; load baseline
    State-->>Sort: snapshot + epoch
    Sort->>Key: evaluate key
    par mutation
        Mut->>State: write lock; change buffer; advance epoch; unlock
    and callback return
        Key-->>Sort: key or exception
    end
    Sort->>State: load epoch
    alt epoch changed or exhausted
        Sort->>Sort: ValueError; discard sorted work
    else unchanged
        Sort->>State: write lock; recheck epoch
        alt changed while acquiring
            Sort->>Sort: ValueError; discard sorted work
        else unchanged
            Sort->>State: write sorted buffer; advance epoch; unlock
        end
    end
```

Required invariants:

1. Snapshot and baseline are captured while holding the same list read guard.
2. Every buffer mutation publishes an epoch change before its write guard is
   released.
3. The final comparison is repeated after acquiring the write guard.
4. Successful sort write-back advances the epoch before releasing that guard,
   so another active sort frame observes it.
5. The 19 legacy marker reachability edges remain observable in the first
   migration, including marker-only no-op/error cases.
6. Append followed by pop advances the epoch twice; final content equality
   cannot erase the mutation observation.
7. Nested frames keep independent baselines and share only object evidence.
8. A frame retains its list until `Drop`; raw addresses are never identities.
9. No TLS state or process-global lock participates.

## Epoch overflow

`mutation_epoch` never wraps. `advance` uses a compare/exchange loop that
saturates at `u64::MAX`. A frame whose baseline is below `MAX` observes the
transition. New sort-frame construction at `MAX` fails closed instead of
accepting a baseline that can no longer advance.

This makes epoch exhaustion explicit and machine-testable; it does not rely on
“2^64 events are unlikely” as a correctness argument.

## Migration seams

1. Introduce `MbListState` around the existing buffer lock with forwarding
   read/write/try-write methods and a non-wrapping epoch.
2. Add `RetainedListHandle` and child-owned `ListSortOperationFrame`.
3. Convert all 19 marker sites. Named guards publish before drop; temporary
   method chains become named critical sections where content changes.
4. Preserve marker-only reachability for empty/equal/error cases.
5. Capture snapshot and baseline together; recheck after acquiring the final
   write guard.
6. Advance the epoch for successful key and no-key sort write-back.
7. Remove both TLS identities, raw list IDs, guard allocation, vector scans,
   and the old watch unit test only after exact-set scanning reaches zero.

The first implementation ticket may touch only `runtime/rc.rs`,
`runtime/list_ops.rs`, and focused list-sort tests. It must not change public
C-ABI signatures, general list behavior, exception text, or unrelated
collection storage.

## Verification gates

- Exact-set gate: two identities and the 32-row current denominator remain
  reconciled until retirement.
- Producer gate: every one of the 19 legacy calls maps to one explicit epoch
  publication edge; no inferred or shifted function owner.
- Reachability gate: equal replacement, empty slice/delete/extend/clear,
  short reverse, repeat-by-one, and repeat type-error behavior are frozen.
- Same-thread gate: a key callback append/pop pair raises
  `ValueError: list modified during sort`.
- Cross-thread gate: a barrier-controlled second worker mutates after snapshot
  and before write-back; sort raises and never overwrites that mutation.
- Acquisition-race gate: mutation between the optimistic check and write-lock
  acquisition is caught by the under-lock recheck.
- Nested gate: two frames for one list both observe a mutation.
- Different-list gate: mutation of list B never invalidates a sort of list A.
- Lifetime gate: releasing all external claims during a sort cannot retire or
  ABA-reuse the retained target.
- Unwind gate: key exception and caught Rust unwind release snapshot, keys,
  retained target, and frame without ambient residue.
- Overflow gate: `MAX-1 -> MAX` invalidates an active frame; new frame at
  `MAX` fails closed.
- Parallelism gate: different lists mutate and sort concurrently with no
  process-global serialization.

## Dependency and controller normalization

- #3001 is a Stage 1 design slice under #2968; it changes no source.
- #2968 must close before Stage 2 #2839 can be dispatched.
- The AGY report required two semantic revisions and one denied exact-read
  retry, so it is not a one-pass ramp sample.
- The controller normalized “content changed” claims for setitem/slice/delete:
  successful call-site reachability includes equal-value and empty-range
  events. The first migration preserves those events rather than silently
  redefining them.
