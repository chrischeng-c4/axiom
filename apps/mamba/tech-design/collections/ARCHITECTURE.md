# collections — architecture (as-is, 2026-07-15)

The four builtin containers (`list`, `dict`, `set`/`frozenset`, `tuple`) and
their C-ABI ops. Source: `src/runtime/{list,dict,set,tuple}_ops.rs`.

## Responsibilities

- Heap reprs + all `mb_{list,dict,set,tuple}_*` C-ABI ops (index/slice/mutate/
  membership/compare) and the name dispatchers `dispatch_{list,dict,set,tuple}_method`
  (list_ops.rs:2228, dict_ops.rs:3118, set_ops.rs:461, tuple_ops.rs:605).
- `DictKey` — the Python-hash key domain for `dict` AND the set/tuple/frozenset
  hashing substrate (dict_ops.rs:506, `to_dict_key` :989).
- Dict insertion-order preservation; set/frozenset algebra; tuple immutability +
  structural hash/ordering; dict views (`keys/items/values`) as set-like.
- Refcount discipline at container boundaries (store_owned on insert,
  release_owned on evict; `DictKey` ptr retain/release on Clone/Drop).
- NOT here: NaN-boxing + SmallVec/lock mechanics (`memory/`), mutation-during-
  iteration guard (`iterators/`, `iter.rs`), numeric key coercion (`numbers/`),
  str/surrogate key encoding (`strings/`).

## Key structures & invariants

| Type | Repr (rc.rs) | Notes |
|---|---|---|
| list | `List(MbRwLock<MbListBuffer>)` :489 | `MbListBuffer::{Generic,Int,Float}`; generic uses `MbList = SmallVec<[MbValue;8]>` :99, while scalar-specialized buffers remain unboxed until an `Any` boundary |
| dict | `Dict(MbRwLock<MbDictMap>)` :490 | `MbDictMap = IndexMap<DictKey,MbValue>` :394 — insertion-ordered |
| tuple | `Tuple(Vec<MbValue>)` :491 | immutable; no lock |
| set | `Set(MbRwLock<MbSet>)` :496 | `MbSet{items:MbList, buckets:FxHashMap<u64,pos>}` :118 |
| frozenset | `FrozenSet(Vec<MbValue>)` :499 | bare Vec — **no hash index**, O(n) membership (set_ops.rs:170) |

- **DictKey Python-hash domain** (dict_ops.rs:506–840): `Str`/`StrCodepoints`
  hash via `dict_string_hash_value` (→ `string_ops::string_hash_value`), **NOT
  Rust `str` hash**. Probe an `IndexMap<DictKey,_>` only via `dict_get_exact_str`
  /`BorrowedDictStrKey` (:860); a raw `.get(&str)` silently misses present keys.
  See `object-model/identity-and-keys.md` §Domain 2 — do not restate.
- **DictKey ownership**: `Instance`/`Tuple`/`FrozenSet` variants carry a raw
  `ptr` retained on Clone (:555)/released on Drop (:607), so `__eq__` breaks
  hash-bucket collisions live.
- **Numeric key collapse is partial**: integral floats normalize to
  `DictKey::Int` in `to_dict_key` (:1021) so `{1:_,1.0:_}` collapses and
  `hash(1.0)==hash(1)`; but `Bool` is a distinct variant (:517) with
  discriminant-based hash + no Int↔Bool eq arm — the `{1:_,True:_}`-collapse
  CPython does at the key layer is **not enforced here** (parity at risk).
- **MbSet consistency**: `set_hash` (rc.rs:146) agrees with `eq_py` (int/bool/
  integral-float share a bucket, resolved by `mb_eq`); mutation ONLY through
  `set_insert`/`set_remove`/`pop_front` (no `DerefMut`) so `buckets` stays synced.
- **List shallow-copy contract**: `list.copy()`, `list(x)` when `x` is a list,
  and `copy.copy(list)` allocate a distinct outer container while preserving
  length, order, and element identity. The copy seam must explicitly invoke
  `MbListBuffer`'s representation-aware conversion/clone; method lookup through
  `Deref<Target=[MbValue]>` is not valid for `Int`/`Float`, whose deref slice is
  deliberately empty. Generic pointer elements gain exactly one owner in the
  destination; scalar-specialized elements require no heap retain.
- Tuple/frozenset are hashable dict keys; list/dict/set are not
  (`unhashable_type_name`, set_ops.rs:448) → `TypeError: unhashable type`.

## Control flow

1. **Construct**: literals → fixed-arity JIT shims `mb_list_new_1..10`/
   `mb_tuple_new_1..8` (inline SmallVec) or `_from_iterable`; `{k:v}` →
   `mb_dict_from_pairs` (:1500); `{..}`/`set(..)` → `mb_set_from_list` (dedup via
   hash index, :53).
2. **dict setitem** (`mb_dict_setitem` :1745): instance-dict/xml stub → reject
   unhashable → `to_dict_key_checked` → `dict_resolve_stored_key` → existing key
   updates in place via `get_index_mut` (**keeps order**) else `map.insert`
   (append) → `bump_dict_version`.
3. **dict delitem** (:1801): `dict_lookup_index` → `shift_remove_index`
   (**order-preserving**, not swap) → bump version. **read**: `to_dict_key` →
   `dict_read_lookup_owned` (:1212) → hit/miss/error.
4. **set op** (`dispatch_set_method` :461): frozenset mutation names rejected
   first (`AttributeError`, :489) → variadic fold for union/intersection/diff.
5. **iterate**: `mb_dict_keys/values/items` snapshot to a list; views via
   `dict_view_make` (:1991) hold the live dict; iterators re-check
   `dict_version`+len each `next` (iter.rs:242) — see `iterators/`.

## CPython-parity semantics (the domain contract)

- **dict insertion order**: preserved. Update-existing keeps position
  (get_index_mut); delete uses `shift_remove` not swap; `popitem` is LIFO
  (`map.pop`, :2851). Re-inserting a deleted key appends at end.
- **str key hashing** is Python-domain, so dict/set iteration order + collision
  behavior track CPython (dict_ops.rs:490); numeric key identity per §invariants.
- **tuple hash** (`mb_tuple_hash` :524): structural over element `mb_hash`
  (value-equal tuples hash equal), truncated `>>17` to MbValue's 48-bit int.
  **frozenset key hash** order-independent via `mb_hash` (to_dict_key :1051).
- **set/frozenset result type follows the LEFT operand** (union etc., :200).
- **dict views are set-like** for `keys`/`items` only (`dict_view_is_setlike`
  :2045); `values` is not; `&|-^`/`==` route through `dict_view_as_set` (:2049).
- **KeyError carries the bare element**, not `repr` (set.remove :123) — repr is
  applied once by `KeyError.__str__`, else double-escaping.

## Known hazards

- Raw `IndexMap<DictKey,_>::get(&str)` — compiles (Equivalent impls exist) but
  hits the wrong bucket → silent None for present keys (dict_ops.rs:494 warning).
- Calling an ambiguous slice method such as `.to_vec()` through an
  `RwLockReadGuard<MbListBuffer>` can resolve through the buffer's
  `Deref<[MbValue]>`. That slice is empty for `Int`/`Float`, so a shallow copy
  silently loses every scalar. Disambiguate the representation-aware
  `MbListBuffer` operation (or clone the enum) before constructing the new
  list, and cover generic, int, float, empty, and nested-reference shapes.
- `MbSet` `DerefMut` absence is deliberate: mutating `items` directly desyncs
  `buckets`; any new mutator must be an inherent method (rc.rs:117).
- `mb_list_sort` drops the lock while the key callable runs and watches for
  concurrent mutation → `ValueError: list modified during sort`
  (`ActiveSortMutationWatch`, list_ops.rs:58,118).
- `to_dict_key` fallback `DictKey::Other(bits-as-string)` (:1083) keys
  non-hashable heap objects by NaN-box identity — wrong-bucket risk for a type
  that should route through `__hash__`.
- Custom-key `__eq__`/`__hash__` re-enters the runtime; setitem resolves the key
  BEFORE taking `write()` (:1770) to avoid re-entrant lock — preserve that order.

## Extension points

- New container method: add the `mb_*` op + a `dispatch_*_method` arm; keep the
  C-ABI thin, push logic into helpers.
- New hashable key type: extend `DictKey` + its Clone/Drop/Hash/PartialEq arms
  AND `to_dict_key` AND `set_hash` (all four must agree or buckets split).
- New view: register `dict_view_class_kind` (:1957) + a `dict_view_make` name;
  set-likeness via `dict_view_is_setlike`.
- New list-copy path: reuse one representation-aware snapshot primitive and
  the borrowed-element ownership constructor. Its proof matrix includes
  `Generic`, `Int`, `Float`, empty, and nested heap references; assert distinct
  outer pointer plus identical nested pointer, then release both owners.
- Perf knobs: `MbList` inline cap (8), `MbSet` bucket layout (coordinate with
  `memory/`); frozenset indexing is the obvious unfilled O(n)→O(1) slot.

## EC surface

- No per-type `core/{list,dict,set,tuple}` fixture lib dir exists; coverage is
  concept-bucketed under `tests/cpython/`: `behavior/core/{compare,dictcomps,
  listcomps,setcomps,container_float_roundtrip,value_equality_inference,iter}`,
  `type/core/{container_element,operator_dispatch}`, plus `errors/core` and
  `real_world`/`3rd-libs`. Record model + dimensions:
  `tests/harness/cpython/conventions/FIXTURE-LAYOUT.md`.
- **C2 perf-pin gap**: `tests/harness/cpython/config/perf/pins/*.toml` are all
  library-level (`protobuf_1513`, `collections_1449`, …); **zero** pin covers
  bare `list`/`dict`/`set`/`tuple` hot paths (#1513 hits dict reads only
  indirectly via protobuf). These hottest primitives are perf-unguarded —
  cross-ref `external-contracts/HARNESS.md`.
