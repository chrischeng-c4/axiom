# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_collections_math_copy_generator_inspect_value_ops"
# subject = "cpython321.test_collections_math_copy_generator_inspect_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_collections_math_copy_generator_inspect_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_collections_math_copy_generator_inspect_value_ops: execute CPython 3.12 seed test_collections_math_copy_generator_inspect_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 253 pass conformance — collections module
# (namedtuple instance with .x/.y, OrderedDict insertion-order keys,
# ChainMap lookup across two maps + shadowing by leftmost, Counter
# .most_common ordering, deque appendleft+append ordering) + math
# module integer-typed surfaces (nan != nan, gcd, lcm, isclose,
# isfinite, isinf, isnan, factorial, trunc, floor, ceil) + copy/
# deepcopy (shallow list shared nested mutation, deep list
# independent mutation, deep dict independent mutation, shallow dict
# shared) + generator advanced (send sends value into yield, throw
# raises in generator caught by try, close cleanly exits via
# GeneratorExit) + inspect hasattr surface (signature/isfunction/
# isclass/ismethod/getmembers) + dis hasattr surface (dis/
# get_instructions/opname/opmap) + keyword module (hasattr kwlist/
# iskeyword/softkwlist + iskeyword True for 'if' and False for non-
# kw) + heapq module (hasattr heappush/heappop/heapify/nlargest/
# nsmallest + heap push/pop ordering, nlargest top-2, nsmallest
# bottom-2). All asserts match between CPython 3.12 and mamba.
import collections
import math
import copy
import inspect
import dis
import keyword
import heapq


Point = collections.namedtuple("Point", ["x", "y"])


_ledger: list[int] = []

# 1) collections
assert Point(1, 2).x == 1; _ledger.append(1)
assert Point(1, 2).y == 2; _ledger.append(1)
assert repr(Point(1, 2)) == "Point(x=1, y=2)"; _ledger.append(1)

def _od_order() -> list:
    od: collections.OrderedDict = collections.OrderedDict()
    od["a"] = 1
    od["b"] = 2
    od["c"] = 3
    return list(od.keys())
assert _od_order() == ["a", "b", "c"]; _ledger.append(1)

assert collections.ChainMap({"a": 1}, {"b": 2})["a"] == 1; _ledger.append(1)
assert collections.ChainMap({"a": 1}, {"b": 2})["b"] == 2; _ledger.append(1)
assert collections.ChainMap({"a": 1}, {"a": 99, "b": 2})["a"] == 1; _ledger.append(1)

assert collections.Counter("abracadabra").most_common(2) == [("a", 5), ("b", 2)]; _ledger.append(1)

def _deque_ops() -> list:
    d: collections.deque = collections.deque([1, 2, 3])
    d.append(4)
    d.appendleft(0)
    return list(d)
assert _deque_ops() == [0, 1, 2, 3, 4]; _ledger.append(1)

# 2) math — integer-typed and predicate surface
assert (math.nan != math.nan) == True; _ledger.append(1)
assert math.gcd(12, 8) == 4; _ledger.append(1)
assert math.lcm(4, 6) == 12; _ledger.append(1)
assert math.isclose(0.1 + 0.2, 0.3) == True; _ledger.append(1)
assert math.isfinite(1.0) == True; _ledger.append(1)
assert math.isinf(math.inf) == True; _ledger.append(1)
assert math.isnan(math.nan) == True; _ledger.append(1)
assert math.factorial(5) == 120; _ledger.append(1)
assert math.trunc(3.9) == 3; _ledger.append(1)
assert math.floor(3.7) == 3; _ledger.append(1)
assert math.ceil(3.1) == 4; _ledger.append(1)

# 3) copy / deepcopy
def _shallow_list_shared():
    a = [1, [2, 3]]
    b = copy.copy(a)
    b[1].append(99)
    return (a, b)
assert _shallow_list_shared() == ([1, [2, 3, 99]], [1, [2, 3, 99]]); _ledger.append(1)

def _deep_list_independent():
    a = [1, [2, 3]]
    b = copy.deepcopy(a)
    b[1].append(99)
    return (a, b)
assert _deep_list_independent() == ([1, [2, 3]], [1, [2, 3, 99]]); _ledger.append(1)

def _deep_dict_independent():
    a = {"k": [1, 2]}
    b = copy.deepcopy(a)
    b["k"].append(99)
    return (a, b)
assert _deep_dict_independent() == ({"k": [1, 2]}, {"k": [1, 2, 99]}); _ledger.append(1)

def _shallow_dict_shared():
    a = {"k": [1, 2]}
    b = copy.copy(a)
    b["k"].append(99)
    return (a, b)
assert _shallow_dict_shared() == ({"k": [1, 2, 99]}, {"k": [1, 2, 99]}); _ledger.append(1)

# 4) generator advanced
def _gen_send():
    def g():
        x = yield 1
        yield x * 2
    gen = g()
    a = next(gen)
    b = gen.send(10)
    return (a, b)
assert _gen_send() == (1, 20); _ledger.append(1)

def _gen_throw() -> str:
    def g():
        try:
            yield 1
        except ValueError:
            yield "caught"
    gen = g()
    next(gen)
    return gen.throw(ValueError("x"))
assert _gen_throw() == "caught"; _ledger.append(1)

def _gen_close() -> str:
    def g():
        try:
            yield 1
            yield 2
        except GeneratorExit:
            pass
    gen = g()
    next(gen)
    gen.close()
    return "closed"
assert _gen_close() == "closed"; _ledger.append(1)

# 5) inspect hasattr surface
assert hasattr(inspect, "signature") == True; _ledger.append(1)
assert hasattr(inspect, "isfunction") == True; _ledger.append(1)
assert hasattr(inspect, "isclass") == True; _ledger.append(1)
assert hasattr(inspect, "ismethod") == True; _ledger.append(1)
assert hasattr(inspect, "getmembers") == True; _ledger.append(1)

# 6) dis hasattr surface
assert hasattr(dis, "dis") == True; _ledger.append(1)
assert hasattr(dis, "get_instructions") == True; _ledger.append(1)
assert hasattr(dis, "opname") == True; _ledger.append(1)
assert hasattr(dis, "opmap") == True; _ledger.append(1)

# 7) keyword module
assert hasattr(keyword, "kwlist") == True; _ledger.append(1)
assert hasattr(keyword, "iskeyword") == True; _ledger.append(1)
assert hasattr(keyword, "softkwlist") == True; _ledger.append(1)
assert keyword.iskeyword("if") == True; _ledger.append(1)
assert keyword.iskeyword("not_a_kw") == False; _ledger.append(1)

# 8) heapq module
assert hasattr(heapq, "heappush") == True; _ledger.append(1)
assert hasattr(heapq, "heappop") == True; _ledger.append(1)
assert hasattr(heapq, "heapify") == True; _ledger.append(1)
assert hasattr(heapq, "nlargest") == True; _ledger.append(1)
assert hasattr(heapq, "nsmallest") == True; _ledger.append(1)

def _heap_ops() -> list:
    h: list = []
    heapq.heappush(h, 3)
    heapq.heappush(h, 1)
    heapq.heappush(h, 2)
    return [heapq.heappop(h), heapq.heappop(h), heapq.heappop(h)]
assert _heap_ops() == [1, 2, 3]; _ledger.append(1)
assert heapq.nlargest(2, [1, 5, 3, 9, 2]) == [9, 5]; _ledger.append(1)
assert heapq.nsmallest(2, [1, 5, 3, 9, 2]) == [1, 2]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_collections_math_copy_generator_inspect_value_ops {sum(_ledger)} asserts")
