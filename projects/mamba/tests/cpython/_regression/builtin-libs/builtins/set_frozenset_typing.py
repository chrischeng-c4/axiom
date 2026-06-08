# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Two related set/frozenset bugs were paired into one fix:
#
# 1. Empty `set()` printed as `{}` from `print(set())` — the top-level
#    `mb_print` and the in-container `print_repr` paths both unconditionally
#    emitted `{...}`, ignoring the empty case. CPython prints `set()` to
#    avoid collision with the empty-dict literal `{}`. (Empty frozenset was
#    already fixed in a prior commit; we apply the same `is_empty()` guard
#    to Set so the two stay symmetric.)
#
# 2. Binary set operators / methods always returned `ObjData::Set`, so
#    `frozenset({1,2}) & {2,3}` came back as `{2}` (a regular set) instead
#    of `frozenset({2})`. CPython's rule is "result type matches the LEFT
#    operand": `frozenset & set -> frozenset`, `set & frozenset -> set`.
#
# Fixes:
#   - `runtime/builtins.rs::mb_print` + the in-list/dict `print_repr` arm:
#     emit `set()` when items are empty.
#   - `runtime/set_ops.rs`: introduce `build_set_like_left(left, items)`
#     which reads the left operand's ObjData and constructs a frozenset
#     iff that operand is one. Apply it in `mb_set_union`,
#     `mb_set_intersection`, `mb_set_difference`,
#     `mb_set_symmetric_difference`.

# Empty set / frozenset print.
print(set())                               # set()
print(repr(set()))                         # 'set()'
print(frozenset())                         # frozenset()
print(repr(frozenset()))                   # 'frozenset()'

# Empty result via binary op preserves the left-operand type.
print(set() | frozenset())                  # set()
print(frozenset() | set())                  # frozenset()

# Frozenset preservation through &|-^ operators.
fs = frozenset([1, 2, 3])
s = {2, 3, 4}

# Left = frozenset → result is frozenset.
print(type(fs & s).__name__)                # frozenset
print(type(fs | s).__name__)                # frozenset
print(type(fs - s).__name__)                # frozenset
print(type(fs ^ s).__name__)                # frozenset

# Left = set → result is set.
print(type(s & fs).__name__)                # set
print(type(s | fs).__name__)                # set
print(type(s - fs).__name__)                # set
print(type(s ^ fs).__name__)                # set

# Same rule for the named methods (.intersection, .union, .difference,
# .symmetric_difference) — type matches self.
print(type(fs.intersection(s)).__name__)    # frozenset
print(type(s.intersection(fs)).__name__)    # set
print(type(fs.union([4, 5])).__name__)      # frozenset

# Empty results in containers — make sure the empty repr survives.
print([set(), frozenset()])                 # [set(), frozenset()]
