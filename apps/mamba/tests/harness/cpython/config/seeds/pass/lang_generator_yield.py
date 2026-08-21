# Operational AssertionPass seed for generator-function and yield
# surfaces.
# Surface: a function with `yield` is a generator — calling it
# returns an iterator; `list(gen())` exhausts it; for-in iterates
# yielded values; generators can be summed; next() advances the
# iterator one step at a time; generator expressions (the `(x for x
# in ...)` paren form) work like generator functions; generator
# expressions plug into sum/any/all/list/zip.
_ledger: list[int] = []


# Basic counting generator
def _count_up_to(n):
    i = 0
    while i < n:
        yield i
        i = i + 1


assert list(_count_up_to(5)) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert list(_count_up_to(0)) == []; _ledger.append(1)
assert list(_count_up_to(1)) == [0]; _ledger.append(1)


# Single-yield generator
def _gen_one():
    yield 42


assert list(_gen_one()) == [42]; _ledger.append(1)


# A generator whose body never yields produces an empty iterator
def _gen_empty():
    if False:
        yield 1


assert list(_gen_empty()) == []; _ledger.append(1)


# Multiple explicit yields in source order
def _gen_three():
    yield "a"
    yield "b"
    yield "c"


assert list(_gen_three()) == ["a", "b", "c"]; _ledger.append(1)


# A generator that yields from a for-loop over range
def _gen_squares(n):
    for i in range(n):
        yield i * i


assert list(_gen_squares(4)) == [0, 1, 4, 9]; _ledger.append(1)
assert list(_gen_squares(0)) == []; _ledger.append(1)
assert list(_gen_squares(1)) == [0]; _ledger.append(1)


# for-in over a generator yields its values one at a time
acc: list[int] = []
for s in _gen_squares(5):
    acc.append(s)
assert acc == [0, 1, 4, 9, 16]; _ledger.append(1)


# sum() over a generator
def _gen_one_to(n):
    i = 1
    while i <= n:
        yield i
        i = i + 1


assert sum(_gen_one_to(5)) == 15; _ledger.append(1)
assert sum(_gen_one_to(10)) == 55; _ledger.append(1)


# next() advances the iterator one step
g = _gen_three()
assert next(g) == "a"; _ledger.append(1)
assert next(g) == "b"; _ledger.append(1)
assert next(g) == "c"; _ledger.append(1)


# Generator expression — paren form behaves like a generator function
assert list(x * 2 for x in [1, 2, 3]) == [2, 4, 6]; _ledger.append(1)
assert list(x + 10 for x in [1, 2, 3]) == [11, 12, 13]; _ledger.append(1)
assert sum(x for x in [1, 2, 3, 4]) == 10; _ledger.append(1)
assert sum(x * x for x in [1, 2, 3]) == 14; _ledger.append(1)


# Generator expression in any()
assert any(x > 5 for x in [1, 2, 3]) == False; _ledger.append(1)
assert any(x > 5 for x in [1, 6, 3]) == True; _ledger.append(1)
# Generator expression in all()
assert all(x > 0 for x in [1, 2, 3]) == True; _ledger.append(1)
assert all(x > 0 for x in [1, -2, 3]) == False; _ledger.append(1)


# Generators paired through zip()
def _gen_a():
    yield "x"
    yield "y"


def _gen_b():
    yield 1
    yield 2


assert list(zip(_gen_a(), _gen_b())) == [("x", 1), ("y", 2)]; _ledger.append(1)


# Generator expression with a predicate (if-clause)
assert list(x for x in [1, 2, 3, 4, 5] if x > 2) == [3, 4, 5]; _ledger.append(1)
assert sum(x for x in [1, 2, 3, 4, 5] if x % 2 == 0) == 6; _ledger.append(1)


# A generator can be exhausted by iter+list, and a fresh call returns
# a new independent iterator
g2 = _count_up_to(3)
first = list(g2)
second = list(_count_up_to(3))
assert first == [0, 1, 2]; _ledger.append(1)
assert second == [0, 1, 2]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_generator_yield {sum(_ledger)} asserts")
