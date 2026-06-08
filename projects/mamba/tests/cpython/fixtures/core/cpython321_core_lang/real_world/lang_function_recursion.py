# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_function_recursion"
# subject = "cpython321.lang_function_recursion"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_function_recursion.py"
# status = "filled"
# ///
"""cpython321.lang_function_recursion: execute CPython 3.12 seed lang_function_recursion"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the function-recursion surface.
# Surface: a function calling itself terminates on its base case and
# returns the canonical reduction value (`fact(0) == 1`, `fact(5) ==
# 120`, `fib(0) == 0`, `fib(10) == 55`); tail-position recursion
# through `power(base, exp)` reduces correctly; recursion over a
# list builds a list back up (`rev(list)` reverses); recursion with
# an accumulator-style result (`count(L, v)`) returns the canonical
# tally; the base-case-only branch returns 0 / [] without entering
# the recursive arm; a function defined inside another function
# (nested def) closes over its parameter and the outer return path
# composes the inner call's result with an additive constant.
# Companion to lang_closures (which covers free-variable capture).
_ledger: list[int] = []

# Classic factorial — base case + multiplicative reduction
def fact(n: int) -> int:
    if n <= 1:
        return 1
    return n * fact(n - 1)

assert fact(0) == 1; _ledger.append(1)
assert fact(1) == 1; _ledger.append(1)
assert fact(5) == 120; _ledger.append(1)
assert fact(6) == 720; _ledger.append(1)
assert fact(10) == 3628800; _ledger.append(1)

# Fibonacci — tree recursion (two recursive calls)
def fib(n: int) -> int:
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

assert fib(0) == 0; _ledger.append(1)
assert fib(1) == 1; _ledger.append(1)
assert fib(5) == 5; _ledger.append(1)
assert fib(10) == 55; _ledger.append(1)
assert fib(15) == 610; _ledger.append(1)

# Recursive power — base case 0 + multiplicative chain
def power(base: int, exp: int) -> int:
    if exp == 0:
        return 1
    return base * power(base, exp - 1)

assert power(2, 0) == 1; _ledger.append(1)
assert power(2, 5) == 32; _ledger.append(1)
assert power(3, 4) == 81; _ledger.append(1)
assert power(5, 3) == 125; _ledger.append(1)

# Recursive list reversal — base case [] + append-after-recursion
def rev(L: list[int]) -> list[int]:
    if not L:
        return []
    return rev(L[1:]) + [L[0]]

assert rev([]) == []; _ledger.append(1)
assert rev([1]) == [1]; _ledger.append(1)
assert rev([1, 2, 3]) == [3, 2, 1]; _ledger.append(1)
assert rev([1, 2, 3, 4]) == [4, 3, 2, 1]; _ledger.append(1)

# Recursive count — base case 0 + conditional increment
def count(L: list[int], v: int) -> int:
    if not L:
        return 0
    rest = count(L[1:], v)
    if L[0] == v:
        return 1 + rest
    return rest

assert count([], 1) == 0; _ledger.append(1)
assert count([1, 2, 1, 3, 1], 1) == 3; _ledger.append(1)
assert count([2, 2, 2], 3) == 0; _ledger.append(1)
assert count([2, 2, 2], 2) == 3; _ledger.append(1)

# Nested-def — outer wraps inner's result with an additive constant
def outer(x: int) -> int:
    def inner(y: int) -> int:
        return y * 2
    return inner(x) + 1

assert outer(0) == 1; _ledger.append(1)
assert outer(5) == 11; _ledger.append(1)
assert outer(10) == 21; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_function_recursion {sum(_ledger)} asserts")
