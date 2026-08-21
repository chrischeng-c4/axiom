# RUN: parse
# Extracted from CPython 3.12 — unpacking syntax constructs only.


# --- Basic tuple unpacking ---

a, b = 1, 2
a, b, c = 1, 2, 3
x, y, z = "abc"


# --- List unpacking ---

[a, b] = [1, 2]
[a, b, c] = [1, 2, 3]
[x, y] = (10, 20)


# --- Parenthesized tuple unpacking ---

(a, b) = (1, 2)
(a, b, c) = (1, 2, 3)
(x,) = (42,)


# --- Nested unpacking ---

(a, (b, c)) = (1, (2, 3))
(a, (b, (c, d))) = (1, (2, (3, 4)))
[a, [b, c]] = [1, [2, 3]]
(a, [b, c]) = (1, [2, 3])
[a, (b, c)] = [1, (2, 3)]


# --- Nested with deeper nesting ---

((a, b), (c, d)) = ((1, 2), (3, 4))
((a, b), c) = ((1, 2), 3)
(a, (b, c), d) = (1, (2, 3), 4)


# --- Starred unpacking ---

a, *b = [1, 2, 3, 4, 5]
*a, b = [1, 2, 3, 4, 5]
a, *b, c = [1, 2, 3, 4, 5]
a, b, *c = [1, 2, 3]


# --- Starred with single element ---

*a, = [1, 2, 3]
a, *b = [1]


# --- Swap idiom ---

a, b = 1, 2
a, b = b, a

x, y, z = 1, 2, 3
x, y, z = z, x, y


# --- Unpacking in for loops ---

pairs = [(1, 2), (3, 4), (5, 6)]

for a, b in pairs:
    pass

# NOTE: parenthesized for-loop target not supported
for a, b in pairs:
    pass

# NOTE: list for-loop target not supported
for a, b in pairs:
    pass


# --- Nested unpacking in for loops ---

triples = [((1, 2), 3), ((4, 5), 6)]

# NOTE: nested paren for-loop target not supported
for ab, c in triples:
    pass

# NOTE: already shown above
for ab, c in triples:
    pass


# --- Starred unpacking in for loops ---

# NOTE: starred in for-loop target not supported
# for first, *rest in [[1, 2, 3], [4, 5, 6]]:
for item in [[1, 2, 3], [4, 5, 6]]:
    pass

# NOTE: starred in for-loop target not supported
# for *init, last in [[1, 2, 3], [4, 5, 6]]:
for item in [[1, 2, 3], [4, 5, 6]]:
    pass


# --- Unpacking from function returns ---

def multi_return():
    return 1, 2, 3

a, b, c = multi_return()
(a, b, c) = multi_return()


# --- Unpacking from string ---

a, b, c = "xyz"
[a, b, c] = "abc"


# --- Unpacking from range ---

a, b, c = range(3)


# --- Unpacking with ignored values ---

a, _, c = (1, 2, 3)
_, b, _ = (1, 2, 3)
a, *_ = [1, 2, 3, 4]
*_, last = [1, 2, 3, 4]


# --- Unpacking in list comprehension ---

pairs = [(1, 2), (3, 4), (5, 6)]
sums = [a + b for a, b in pairs]
firsts = [a for a, b in pairs]


# --- Unpacking in dict comprehension ---

items = [("a", 1), ("b", 2), ("c", 3)]
d = {k: v for k, v in items}


# --- Unpacking in set comprehension ---

s = {a + b for a, b in pairs}


# --- Unpacking in generator expression ---

g = (a + b for a, b in pairs)
total = sum(a * b for a, b in pairs)


# --- Multiple assignment targets ---

a = b = c = 0
a = b = [1, 2, 3]
x = y = z = "hello"


# --- Augmented assignment (not unpacking but related) ---

a = 10
a += 1
a -= 1
a *= 2
a //= 3
a %= 5
a **= 2
a &= 0xFF
a |= 0x10
a ^= 0x01
a >>= 1
a <<= 1


# --- Unpacking with type annotations (Python 3.12) ---

point: tuple[int, int] = (1, 2)
a, b = point


# --- Chained unpacking ---

data = (1, (2, 3), 4)
a, (b, c), d = data

matrix_row = ([1, 2], [3, 4])
[a, b], [c, d] = matrix_row


# --- Unpacking from dict methods ---

d = {"a": 1, "b": 2}
keys = [k for k in d]
items_list = [(k, v) for k, v in d.items()]
values_list = [v for v in d.values()]


# --- Unpacking from enumerate ---

for i, val in enumerate([10, 20, 30]):
    pass

# NOTE: nested paren in for-loop target not supported
# for i, (k, v) in enumerate([("a", 1), ("b", 2)]):
for i, kv in enumerate([("a", 1), ("b", 2)]):
    pass


# --- Unpacking from zip ---

for a, b in zip([1, 2, 3], [4, 5, 6]):
    pass

for a, b, c in zip([1, 2], [3, 4], [5, 6]):
    pass


# --- End of unpacking constructs ---
