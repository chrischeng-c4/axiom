# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""collections (builtin-libs): documented exception paths.

This is a thin alias to the std-libs/collections coverage; the builtin
type variant exists as a directory in our tree, so we mirror the key
exception paths here for S1 coverage.
"""

from collections import OrderedDict, deque, Counter


# Missing key in OrderedDict raises KeyError.
od: OrderedDict = OrderedDict()
try:
    od["missing"]
    print("od_missing: no_raise")
except KeyError as e:
    print("od_missing:", type(e).__name__, str(e)[:60])


# move_to_end on missing raises KeyError.
try:
    od.move_to_end("nope")
    print("od_move: no_raise")
except KeyError as e:
    print("od_move:", type(e).__name__, str(e)[:60])


# deque.pop on empty raises IndexError.
d: deque = deque()
try:
    d.pop()
    print("deque_empty: no_raise")
except IndexError as e:
    print("deque_empty:", type(e).__name__, str(e)[:60])


# Counter subtract with non-mapping/non-iterable.
c: Counter = Counter("aab")
try:
    c.subtract(42)  # type: ignore[arg-type]
    print("ctr_subtract_int: no_raise")
except TypeError as e:
    print("ctr_subtract_int:", type(e).__name__, str(e)[:60])


# Counter most_common with negative n: returns empty list (no raise).
print("ctr_neg_n:", c.most_common(-1))


# Happy: Counter math.
c1 = Counter(a=2, b=1)
c2 = Counter(a=1)
print("counter_sub:", c1 - c2)
