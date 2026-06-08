# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_lambda_use"
# subject = "cpython321.lang_lambda_use"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_lambda_use.py"
# status = "filled"
# ///
"""cpython321.lang_lambda_use: execute CPython 3.12 seed lang_lambda_use"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for lambda usage patterns beyond
# lang_lambda_expressions (basic single-arg form). Surface: lambda as
# key= callback for sorted (by str length, by reversed value, by tuple
# field); lambda inside filter / map (single iterable); lambda assigned
# to a local name and called like a regular function; lambda with a
# default argument; conditional-expression lambda (returns one of two
# values based on a predicate); lambda as dict value building a small
# dispatch table; lambdas built inside a list literal (NOT a closure
# — each lambda captures only its own arg); any() / all() with map +
# lambda predicate; nested lambdas — outer returns inner that closes
# over outer's arg (currying).
_ledger: list[int] = []

# sorted with key= lambda — by string length
assert sorted(["bbb", "a", "cc"], key=lambda s: len(s)) == ["a", "cc", "bbb"]; _ledger.append(1)
# sorted descending via key=-x
assert sorted([1, 2, 3], key=lambda x: -x) == [3, 2, 1]; _ledger.append(1)
# sorted dict items by value (kv[1])
assert sorted({"a": 3, "b": 1, "c": 2}.items(), key=lambda kv: kv[1]) == [("b", 1), ("c", 2), ("a", 3)]; _ledger.append(1)

# filter with lambda predicate
assert list(filter(lambda x: x > 2, [1, 2, 3, 4])) == [3, 4]; _ledger.append(1)
# Predicate that never matches → empty result
assert list(filter(lambda x: x > 100, [1, 2, 3])) == []; _ledger.append(1)
# Predicate that always matches → all elements
assert list(filter(lambda x: True, [1, 2, 3])) == [1, 2, 3]; _ledger.append(1)

# map with lambda transform (single iterable)
assert list(map(lambda x: x * 2, [1, 2, 3])) == [2, 4, 6]; _ledger.append(1)
assert list(map(lambda x: x + 100, [1, 2, 3])) == [101, 102, 103]; _ledger.append(1)
assert list(map(lambda s: s.upper(), ["a", "b"])) == ["A", "B"]; _ledger.append(1)

# Lambda assigned to a local name; behaves just like a regular function
add = lambda a, b: a + b
assert add(3, 4) == 7; _ledger.append(1)
mul = lambda a, b: a * b
assert mul(5, 6) == 30; _ledger.append(1)

# Lambda with a default argument
greet = lambda name, prefix="Hello": prefix + " " + name
assert greet("alice") == "Hello alice"; _ledger.append(1)
assert greet("bob", "Hi") == "Hi bob"; _ledger.append(1)

# Conditional-expression lambda — returns one of two values based on a
# predicate (this is the canonical lambda + ternary combination)
choose = lambda x: "big" if x > 10 else "small"
assert choose(20) == "big"; _ledger.append(1)
assert choose(5) == "small"; _ledger.append(1)

# Lambda as dict value — dispatch table by op-name string
ops: dict = {
    "add": lambda a, b: a + b,
    "sub": lambda a, b: a - b,
    "mul": lambda a, b: a * b,
}
assert ops["add"](2, 3) == 5; _ledger.append(1)
assert ops["sub"](10, 4) == 6; _ledger.append(1)
assert ops["mul"](3, 4) == 12; _ledger.append(1)

# Lambdas inside a list literal — each lambda captures only its own
# argument (no closure over a loop variable)
funcs = [lambda x: x * 2, lambda x: x + 1, lambda x: x ** 2]
assert funcs[0](5) == 10; _ledger.append(1)
assert funcs[1](5) == 6; _ledger.append(1)
assert funcs[2](5) == 25; _ledger.append(1)

# any() with map + lambda predicate
assert any(map(lambda x: x > 3, [1, 2, 4])) == True; _ledger.append(1)
assert any(map(lambda x: x > 100, [1, 2, 3])) == False; _ledger.append(1)

# all() with map + lambda predicate
assert all(map(lambda x: x > 0, [1, 2, 3])) == True; _ledger.append(1)
assert all(map(lambda x: x > 5, [1, 6, 7])) == False; _ledger.append(1)

# Nested lambdas — outer returns inner that closes over outer's arg
# (the classic curry pattern: `add5 = curry(5)`, then `add5(10) == 15`)
outer = lambda x: (lambda y: x + y)
add5 = outer(5)
assert add5(10) == 15; _ledger.append(1)
assert add5(20) == 25; _ledger.append(1)
add100 = outer(100)
assert add100(1) == 101; _ledger.append(1)

# sorted descending with reverse= and lambda key combined
assert sorted([1, 2, 3], key=lambda x: x, reverse=True) == [3, 2, 1]; _ledger.append(1)

# min / max with key= lambda
assert min([1, 2, 3], key=lambda x: -x) == 3; _ledger.append(1)
assert max(["a", "bb", "ccc"], key=lambda s: len(s)) == "ccc"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_lambda_use {sum(_ledger)} asserts")
