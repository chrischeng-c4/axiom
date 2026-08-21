# RUN: parse
# Extracted from CPython Lib/test/test_listcomps.py — list comprehension syntax constructs only.


# --- Basic list comprehension ---

[x for x in range(10)]
[x for x in [1, 2, 3, 4, 5]]
[x for x in "hello"]
[x for x in (1, 2, 3)]
[x for x in {1, 2, 3}]
[x for x in {"a": 1, "b": 2}]


# --- List comprehension with condition ---

[x for x in range(10) if x > 5]
[x for x in range(20) if x % 2 == 0]
[x for x in range(100) if x % 3 == 0 and x % 5 == 0]
[x for x in "hello world" if x != " "]
[x for x in range(50) if x > 10 if x < 40]
[x for x in range(100) if x % 2 == 0 if x % 3 == 0 if x % 5 == 0]


# --- List comprehension with function application ---

[abs(x) for x in [-3, -2, -1, 0, 1, 2, 3]]
[len(s) for s in ["hello", "world", "!"]]
[s.upper() for s in ["hello", "world"]]
[s.strip() for s in ["  a  ", " b ", "  c"]]
[int(s) for s in ["1", "2", "3"]]
[str(x) for x in range(5)]
[float(x) for x in range(5)]
[hex(x) for x in range(16)]
[bin(x) for x in range(8)]
[chr(x) for x in range(65, 91)]
[ord(c) for c in "ABCDE"]
[type(x).__name__ for x in [1, "a", 2.0, None, True]]
[repr(x) for x in [1, "hello", [1, 2]]]
[sorted(lst) for lst in [[3, 1, 2], [6, 4, 5]]]
[list(reversed(lst)) for lst in [[1, 2, 3], [4, 5, 6]]]


# --- Nested list comprehensions ---

[[y for y in range(x)] for x in range(5)]
[x for row in [[1, 2, 3], [4, 5, 6], [7, 8, 9]] for x in row]
[x for xs in [[1, 2], [3, 4], [5, 6]] for x in xs]
[(x, y) for x in range(3) for y in range(3)]
[x * y for x in range(1, 4) for y in range(1, 4)]
[x + y for x in [10, 20, 30] for y in [1, 2, 3]]
[c for word in ["hello", "world"] for c in word]

# Triple nesting
[x for xss in [[[1, 2], [3]], [[4, 5]]] for xs in xss for x in xs]


# --- Multiple conditions in nested comprehensions ---

[(x, y) for x in range(5) for y in range(5) if x != y]
[(x, y) for x in range(10) if x % 2 == 0 for y in range(10) if y % 3 == 0]
[x + y for x in range(5) if x > 1 for y in range(5) if y > 2 if x + y < 7]


# --- Complex expressions ---

[x * x for x in range(10)]
[x ** 2 for x in range(10)]
[x ** 3 for x in range(5)]
[2 ** x for x in range(8)]
[x + 1 for x in range(10)]
[x * 2 + 1 for x in range(10)]
[(x + 1) * (x - 1) for x in range(2, 10)]

[x if x > 0 else -x for x in [-3, -1, 0, 2, 4]]
[x if x % 2 == 0 else x * 10 for x in range(10)]
["even" if x % 2 == 0 else "odd" for x in range(5)]
[None if x == 0 else 1 / x for x in range(-3, 4) if x != 0]

[(lambda n: n * 2)(x) for x in range(5)]
[f(x) for f in [abs, float, str] for x in [-1, 0, 1]]


# --- With walrus operator ---

[y for x in range(10) if (y := x * x) > 20]
[z for s in ["hello", "hi", "hey"] if (z := len(s)) > 2]
[clean for raw in ["  a  ", "", " b "] if (clean := raw.strip())]
[result for x in range(20) if (result := x ** 2 + x + 1) > 100]


# --- Tuple unpacking in for ---

[(a, b) for a, b in [(1, 2), (3, 4), (5, 6)]]
[a + b for a, b in [(1, 2), (3, 4), (5, 6)]]
[a * b for a, b in zip(range(5), range(5))]
[f"{k}={v}" for k, v in {"a": 1, "b": 2}.items()]
[k for k, v in {"a": 1, "b": 2, "c": 3}.items() if v > 1]
[v for k, v in enumerate("abc")]
[f"{i}: {x}" for i, x in enumerate(["a", "b", "c"])]

# Triple unpacking
[a + b + c for a, b, c in [(1, 2, 3), (4, 5, 6)]]
[f"{a}-{b}-{c}" for a, b, c in zip("abc", "def", "ghi")]

# Star unpacking
# NOTE: star in for-loop target inside list comp not supported
# [first for first, *_ in [(1, 2, 3), (4, 5, 6)]]
# [rest for _, *rest in [(1, 2, 3), (4, 5, 6)]]

# Nested tuple unpacking
# NOTE: nested paren tuple in for target not supported in list comp
# [a + d for (a, b), (c, d) in [((1, 2), (3, 4)), ((5, 6), (7, 8))]]


# --- String operations in list comprehensions ---

[c.upper() for c in "hello"]
[word.capitalize() for word in "hello world".split()]
[s.replace("a", "b") for s in ["cat", "hat", "mat"]]
[s[::-1] for s in ["hello", "world"]]
[s.zfill(5) for s in ["1", "22", "333"]]
[s.center(10) for s in ["a", "bb", "ccc"]]
[s.encode("utf-8") for s in ["hello", "world"]]


# --- List comprehension with complex data structures ---

[{"name": n, "len": len(n)} for n in ["Alice", "Bob", "Charlie"]]
[{x} for x in range(5)]
[[x, x * 2] for x in range(5)]
[(x,) for x in range(5)]
[frozenset([x, x + 1]) for x in range(5)]
[(*pair,) for pair in [(1, 2), (3, 4)]]
# NOTE: dict unpacking {**d, ...} not supported
# [{**d, "extra": True} for d in [{"a": 1}, {"b": 2}]]


# --- Scope isolation ---

x = "outer"
result = [x for x in range(5)]
x  # still "outer"

items = [1, 2, 3]
result = [items for items in [items]]


# --- Nested list comprehension as expression ---

matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
transposed = [[row[i] for row in matrix] for i in range(3)]
flattened = [x for row in matrix for x in row]
diagonal = [matrix[i][i] for i in range(len(matrix))]


# --- List comprehension with membership test ---

[x for x in range(20) if x in {0, 5, 10, 15}]
[x for x in "hello world" if x not in "aeiou"]
[x for x in range(10) if x not in [2, 4, 6, 8]]


# --- List comprehension in various contexts ---

len([x for x in range(10)])
sum([x for x in range(10)])
max([x for x in range(10)])
min([x for x in range(10)])
sorted([x for x in [3, 1, 4, 1, 5]])
",".join([str(x) for x in range(5)])
list(reversed([x for x in range(5)]))
tuple([x for x in range(5)])
set([x for x in range(10)])

# Assignment
result = [x * 2 for x in range(5)]
a, b, c, d, e = [x for x in range(5)]
first, *rest = [x for x in range(10)]

# As function argument
print([x for x in range(3)])
dict([(k, v) for k, v in enumerate("abc")])


# --- Boolean list comprehensions ---

[bool(x) for x in [0, 1, "", "a", None, [], [1]]]
[not x for x in [True, False, True]]
[x and y for x, y in [(True, True), (True, False), (False, True)]]
[x or y for x, y in [(True, True), (True, False), (False, False)]]


# --- List comprehension with exception-safe patterns ---

[x for x in range(10) if x != 0]
[1 / x for x in range(1, 10)]


# --- Empty and single-element ---

[x for x in []]
[x for x in range(0)]
[x for x in [42]]
[x for x in range(1)]
