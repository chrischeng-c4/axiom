"""Behavior contract for language comprehensions.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: list comprehension basic
assert [x ** 2 for x in range(6)] == [0, 1, 4, 9, 16, 25]

# Rule 2: list comp with filter
assert [x for x in range(20) if x % 3 == 0] == [0, 3, 6, 9, 12, 15, 18]

# Rule 3: list comp over string
assert [c.upper() for c in "hello"] == ["H", "E", "L", "L", "O"]

# Rule 4: nested list comp — flattens matrix
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
flat = [v for row in matrix for v in row]
assert flat == [1, 2, 3, 4, 5, 6, 7, 8, 9], f"flatten = {flat!r}"

# Rule 5: dict comprehension
d = {str(i): i * i for i in range(5)}
assert d == {"0": 0, "1": 1, "2": 4, "3": 9, "4": 16}, f"dict comp = {d!r}"

# Rule 6: dict comp inversion (assuming unique values)
orig = {"a": 1, "b": 2, "c": 3}
inv = {v: k for k, v in orig.items()}
assert inv == {1: "a", 2: "b", 3: "c"}, f"inv = {inv!r}"

# Rule 7: set comprehension deduplicates
words = ["hello", "world", "hello", "python"]
unique_lens = {len(w) for w in words}
assert unique_lens == {5, 6}, f"set comp = {unique_lens!r}"

# Rule 8: generator expression is lazy
counter = 0
def _inc(x: int) -> int:
    global counter
    counter += 1
    return x
gen = (_inc(i) for i in range(100))
assert counter == 0, "gen expr evaluated eagerly"
next(gen)
assert counter == 1, f"gen expr step = {counter!r}"

# Rule 9: comprehension scope isolates loop variable
x = 42
result = [x for x in range(3)]
assert x == 42, f"outer x leaked: {x!r}"  # outer x unchanged
assert result == [0, 1, 2], f"result = {result!r}"

# Rule 10: nested comp with condition
pairs = [(x, y) for x in range(4) for y in range(4) if x != y and x + y == 4]
assert pairs == [(1, 3), (3, 1)], f"pairs = {pairs!r}"

# Rule 11: walrus operator in comprehension (Python 3.8+)
_data = [1, 2, 3, 4, 5, 6]
results = [y for x in _data if (y := x * x) > 10]
assert results == [16, 25, 36], f"walrus comp = {results!r}"

print("behavior OK")
