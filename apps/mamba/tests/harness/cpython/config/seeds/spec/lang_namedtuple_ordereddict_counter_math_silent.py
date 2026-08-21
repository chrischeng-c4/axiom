# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `namedtuple` indexed access
# `Point(5, 6)[0]` (the documented "namedtuple is a tuple — integer
# indexing returns the positional field" — mamba returns None),
# `Point._fields` (the documented "namedtuple class attribute exposes
# the tuple of field names" — mamba returns None),
# `Point(1, 2)._asdict()` (the documented "_asdict returns an OrderedDict
# of fields" — mamba raises AttributeError because the per-instance
# method is missing), `Point(1, 2)._replace(x=99)` (the documented
# "_replace returns a new namedtuple with the named fields replaced" —
# mamba raises AttributeError), `OrderedDict.move_to_end` (the
# documented "OrderedDict reorders an existing key to either end" —
# mamba raises AttributeError because OrderedDict instances do not
# expose the method), `OrderedDict.popitem` (the documented
# "popitem returns and removes the last-inserted (key, value) pair" —
# mamba raises AttributeError), `Counter + Counter` (the documented
# "Counter addition merges with summed counts" — mamba returns an
# empty mapping), `math.pi` constant (the documented "module-level
# float constant" — mamba returns a boxed-handle int), `math.sqrt(4)`
# (the documented "returns float square root" — mamba returns a
# boxed-handle int), and `inspect.Parameter` symbol (the documented
# "inspect module exposes Parameter class" — mamba's inspect module
# does not expose the attribute, so hasattr returns False).
# Ten-pack pinned to atomic 253.
#
# Behavioral edges that CONFORM on mamba (collections — namedtuple
# .x/.y instance attr and repr; OrderedDict insertion-order keys list;
# ChainMap lookup of each map + leftmost-wins shadowing; Counter
# .most_common ordering; deque appendleft+append ordering. math —
# nan != nan, gcd, lcm, isclose, isfinite, isinf, isnan, factorial,
# trunc, floor, ceil. copy — shallow vs deep across list+dict.
# generator — send into yield, throw caught by try, close via
# GeneratorExit. inspect — hasattr signature/isfunction/isclass/
# ismethod/getmembers. dis — hasattr dis/get_instructions/opname/
# opmap. keyword — hasattr kwlist/iskeyword/softkwlist + iskeyword
# True for 'if' / False for non-kw. heapq — hasattr surface 5 +
# heap push/pop, nlargest, nsmallest top-2 ordering) are covered in
# the matching pass fixture
# `test_collections_math_copy_generator_inspect_value_ops`.
import collections
import math
import inspect
from typing import Any


Point = collections.namedtuple("Point", ["x", "y"])


_ledger: list[int] = []

# 1) namedtuple integer index — must return positional field
#    (mamba: Point(5, 6)[0] returns None)
assert Point(5, 6)[0] == 5; _ledger.append(1)

# 2) namedtuple._fields — class attribute holding field-name tuple
#    (mamba: returns None)
assert Point._fields == ("x", "y"); _ledger.append(1)

# 3) namedtuple._asdict — instance method returning OrderedDict
#    (mamba: AttributeError 'Point' object has no attribute '_asdict')
def _nt_asdict() -> Any:
    try:
        return dict(Point(1, 2)._asdict())
    except AttributeError:
        return None
assert _nt_asdict() == {"x": 1, "y": 2}; _ledger.append(1)

# 4) namedtuple._replace — instance method returning new namedtuple
#    (mamba: AttributeError 'Point' object has no attribute '_replace')
def _nt_replace() -> Any:
    try:
        r = Point(1, 2)._replace(x=99)
        return (r.x, r.y)
    except AttributeError:
        return None
assert _nt_replace() == (99, 2); _ledger.append(1)

# 5) OrderedDict.move_to_end — reorder existing key to end
#    (mamba: AttributeError on instance)
def _od_move_to_end() -> Any:
    od: collections.OrderedDict = collections.OrderedDict(
        [("a", 1), ("b", 2), ("c", 3)]
    )
    try:
        od.move_to_end("a")
    except AttributeError:
        return None
    return list(od.keys())
assert _od_move_to_end() == ["b", "c", "a"]; _ledger.append(1)

# 6) OrderedDict.popitem — remove and return last-inserted pair
#    (mamba: AttributeError on instance)
def _od_popitem() -> Any:
    od: collections.OrderedDict = collections.OrderedDict(
        [("a", 1), ("b", 2), ("c", 3)]
    )
    try:
        return od.popitem()
    except AttributeError:
        return None
assert _od_popitem() == ("c", 3); _ledger.append(1)

# 7) Counter + Counter — merge with summed counts
#    (mamba: returns {})
def _counter_add() -> dict:
    c = collections.Counter({"a": 3, "b": 1})
    d = collections.Counter({"a": 1, "c": 1})
    return dict(c + d)
assert _counter_add() == {"a": 4, "b": 1, "c": 1}; _ledger.append(1)

# 8) math.pi — module-level float constant
#    (mamba: returns a boxed-handle int, not 3.141592653589793)
assert math.pi == 3.141592653589793; _ledger.append(1)

# 9) math.sqrt(4) — must return float 2.0
#    (mamba: returns a boxed-handle int)
def _math_sqrt() -> Any:
    return math.sqrt(4)
assert _math_sqrt() == 2.0; _ledger.append(1)

# 10) inspect.Parameter — class symbol exposed on inspect module
#     (mamba: hasattr returns False — symbol missing)
assert hasattr(inspect, "Parameter") == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_namedtuple_ordereddict_counter_math_silent {sum(_ledger)} asserts")
