# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""data_structures (catch-all): exception paths for stdlib data
structures (CPython 3.12 oracle)."""

from collections import deque, ChainMap, defaultdict


# deque indexing OOR raises IndexError.
d = deque([1, 2, 3])
try:
    d[5]
    print("deque_oor: no_raise")
except IndexError as e:
    print("deque_oor:", type(e).__name__, str(e)[:60])


# Bounded deque silently drops oldest.
b: deque = deque(maxlen=2)
b.append(1); b.append(2); b.append(3)
print("bounded:", list(b))


# ChainMap.pop only pops from first map; missing in first raises KeyError.
cm = ChainMap({"a": 1}, {"b": 2})
try:
    cm.pop("b")
    print("cm_pop_b: no_raise")
except KeyError as e:
    print("cm_pop_b:", type(e).__name__, str(e)[:60])


# defaultdict with non-callable factory raises TypeError at construction.
try:
    defaultdict("x")  # type: ignore[arg-type]
    print("bad_factory: no_raise")
except TypeError as e:
    print("bad_factory:", type(e).__name__, str(e)[:60])


# Happy: defaultdict supplies missing.
dd: defaultdict = defaultdict(list)
dd["k"].append(1)
print("dd:", dict(dd))
