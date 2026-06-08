# lang_forward_ref.py — #3364 axis-1 forward-ref + custom-type annotation seed.
#
# Exercises:
#   1. Custom-type annotation on a function argument (user-defined class)
#   2. Custom-type annotation on a function return
#   3. Forward-reference string annotation on an attribute (`"Linked | None"`)
#   4. Forward-reference string annotation on a return type (`"Linked"`)
#   5. Self-reference forward annotation inside a method
#
# Distilled from #2939 (issue body); kept narrow to the items that mamba
# accepts today.
#
# Mamba quirks (tracked separately):
#   * `from __future__ import annotations` deferred resolution (#3508).
#
# Contract: AssertionError → Fail; MAMBA_ASSERTION_PASS → AssertionPass.

_ledger: list[int] = []

# (1) Custom-type annotation on a function argument
class _Node:
    def __init__(self, value):
        self.value = value

def _get_value(n: _Node) -> int:
    return n.value

_n = _Node(42)
assert _get_value(_n) - 42 == 0, (
    f"function with custom-type arg, got {_get_value(_n)!r}"
)
_ledger.append(1)

# (2) Custom-type annotation on a function return
def _make_node(v: int) -> _Node:
    return _Node(v)

_n2 = _make_node(7)
assert _n2.value - 7 == 0, f"function with custom-type return, got {_n2.value!r}"
_ledger.append(1)

# (3) Forward-ref string annotation on an attribute
class _Linked:
    def __init__(self, val):
        self.val = val
        self.next: "_Linked | None" = None

_a = _Linked(1)
_b = _Linked(2)
_a.next = _b
assert _a.next is not None, "forward-ref attribute initialised then assigned"
_ledger.append(1)
assert _a.next.val - 2 == 0, (
    f"forward-ref attribute readback, got {_a.next.val!r}"
)
_ledger.append(1)

# (4) Forward-ref string annotation on a return type
def _make_linked(name) -> "_Linked":
    return _Linked(name)

_l = _make_linked("x")
assert _l.val == "x", f"forward-ref return type readback, got {_l.val!r}"
_ledger.append(1)

# (5) Self-reference forward annotation inside a method
class _Tree:
    def __init__(self, value):
        self.value = value
        self.left: "_Tree | None" = None
        self.right: "_Tree | None" = None

    def add_left(self, child: "_Tree") -> "_Tree":
        self.left = child
        return self

_t = _Tree(1)
_chained = _t.add_left(_Tree(2))
# add_left returns self (chained for builder pattern)
assert _chained is _t, "self-ref method returns self for chaining"
_ledger.append(1)

assert _t.left is not None, "self-ref method assigned left subtree"
_ledger.append(1)
assert _t.left.value - 2 == 0, (
    f"self-ref subtree value, got {_t.left.value!r}"
)
_ledger.append(1)

# (6) Linked-list traversal via forward-ref attribute
_c = _Linked(10)
_a.next = _c  # rebind from 2 → 10
assert _a.next.val - 10 == 0, (
    f"forward-ref rebind readback, got {_a.next.val!r}"
)
_ledger.append(1)

# (7) Custom-type-annotated factory that returns the custom type
def _new_tree(v: int) -> "_Tree":
    return _Tree(v)

_t2 = _new_tree(99)
assert _t2.value - 99 == 0, f"forward-ref factory return, got {_t2.value!r}"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_forward_ref {sum(_ledger)} asserts")
