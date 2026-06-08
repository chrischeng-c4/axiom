# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/comprehension_scope: comprehension semantics (CPython 3.12 oracle)."""

# Generic language behavior checks.
assert 1 + 1 == 2
assert "a" + "b" == "ab"
assert isinstance(True, int)
assert type([]) is list
assert type({}) is dict
assert list(range(3)) == [0, 1, 2]


# A comprehension reads names from the ENCLOSING scope (closure read).
g_outer = "Global variable"
assert {k: g_outer for k in range(3)} == {0: g_outer, 1: g_outer, 2: g_outer}


def reads_local():
    v = "Local variable"
    out = {k: v for k in range(3)}
    # The enclosing local is read, not consumed by the comprehension.
    assert v == "Local variable"
    return out


assert reads_local() == {0: "Local variable", 1: "Local variable", 2: "Local variable"}


# Temp-variable assignment idiom: `for j in [expr]` binds a per-iteration name.
assert {j: j * j for i in range(4) for j in [i + 1]} == {1: 1, 2: 4, 3: 9, 4: 16}
assert {j * k for i in range(4) for j in [i + 1] for k in [j + 1]} == {2, 6, 12, 20}
# Tuple unpacking in the for-target.
assert [j * k for i in range(3) for j, k in [(i + 1, i + 2)]] == [2, 6, 12]


# Multiple for-clauses act as a nested cross product, with a trailing filter.
assert {k: v for k in range(4) for v in range(4) if k == v} == {0: 0, 1: 1, 2: 2, 3: 3}


# Key sub-expression is evaluated before the value sub-expression, left to right.
calls = []


def rec(tag, value):
    calls.append(tag)
    return value


pairs = {rec("k", k): rec("v", v) for k, v in zip("ab", "xy")}
assert pairs == {"a": "x", "b": "y"}
assert calls == ["k", "v", "k", "v"]


# Star-unpacking is allowed in the iterable expression.
assert {i: i * i for i in [*range(4)]} == {0: 0, 1: 1, 2: 4, 3: 9}
assert {i: i * i for i in (*range(4),)} == {0: 0, 1: 1, 2: 4, 3: 9}


# A nested comprehension reads the outer for-clause variable.
assert [[r * c for c in range(3)] for r in range(2)] == [[0, 0, 0], [0, 1, 2]]


# PEP 572: a walrus target inside a comprehension LEAKS to the enclosing scope.
def walrus_leaks():
    vals = [(y := n * 2) for n in range(4)]
    # y escaped the comprehension and holds the final assignment.
    return vals, y


assert walrus_leaks() == ([0, 2, 4, 6], 6)

print("behavior OK")
