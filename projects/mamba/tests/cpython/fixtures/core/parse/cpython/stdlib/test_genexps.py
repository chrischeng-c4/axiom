# RUN: parse
# Extracted from CPython Lib/test/test_genexps.py — generator expression syntax constructs only.


# --- Basic generator expression ---

g = (x for x in range(10))
next(g)
list(g)

g2 = (x for x in [1, 2, 3, 4, 5])
tuple(g2)


# --- Generator expression with condition ---

evens = (x for x in range(20) if x % 2 == 0)
list(evens)

positives = (x for x in [-2, -1, 0, 1, 2, 3] if x > 0)
list(positives)

non_empty = (s for s in ["", "a", "", "b", "c"] if s)
list(non_empty)

big = (x for x in range(100) if x > 90 if x % 2 == 0)
list(big)


# --- Generator expression with function application ---

squares = (x * x for x in range(10))
list(squares)

lengths = (len(s) for s in ["hello", "world", "!"])
list(lengths)

uppers = (s.upper() for s in ["hello", "world"])
list(uppers)

types = (type(x) for x in [1, "a", 2.0, None])
list(types)

mapped = (abs(x) for x in [-3, -2, -1, 0, 1, 2, 3])
list(mapped)

converted = (int(s) for s in ["1", "2", "3"])
list(converted)

transformed = (str(x) + "!" for x in range(5))
list(transformed)


# --- Nested generator expressions ---

flat = (x for xs in [[1, 2], [3, 4], [5, 6]] for x in xs)
list(flat)

matrix_flat = (cell for row in [[1, 2, 3], [4, 5, 6], [7, 8, 9]] for cell in row)
list(matrix_flat)

deep = (x for xss in [[[1, 2], [3]], [[4, 5]]] for xs in xss for x in xs)
list(deep)

product = ((x, y) for x in range(3) for y in range(3))
list(product)

filtered_product = ((x, y) for x in range(5) for y in range(5) if x != y)
list(filtered_product)

triangular = ((i, j) for i in range(4) for j in range(i + 1))
list(triangular)


# --- Multiple conditions ---

multi_cond = (x for x in range(100) if x % 2 == 0 if x % 3 == 0)
list(multi_cond)

chained_filter = (x for x in range(50) if x > 10 if x < 40 if x % 7 == 0)
list(chained_filter)


# --- As function arguments ---

sum(x for x in range(10))
sum(x * x for x in range(10))
max(x for x in [3, 1, 4, 1, 5, 9])
min(x for x in [3, 1, 4, 1, 5, 9])
any(x > 3 for x in range(5))
all(x > 0 for x in range(1, 10))
sorted(x for x in [3, 1, 4, 1, 5])

list(x for x in range(5))
tuple(x for x in range(5))
set(x % 3 for x in range(10))
frozenset(x % 4 for x in range(12))

",".join(str(x) for x in range(5))
" ".join(s.strip() for s in ["  a  ", " b ", "c"])

dict((x, x * x) for x in range(5))
dict((k, v) for k, v in [("a", 1), ("b", 2)])


# --- Genexp as sole argument (no extra parens needed) ---

sum(x for x in range(10))
min(x for x in [5, 3, 8])
max(x for x in [5, 3, 8])

# With extra parens (also valid)
sum((x for x in range(10)))
list((x for x in range(5)))


# --- With walrus operator ---

results = list(y for x in range(10) if (y := x * x) > 20)

filtered = list(z for s in ["hello", "hi", "hey", "h"]
                if (z := len(s)) > 1)

processed = list(r for item in ["abc", "de", "f", "ghij"]
                 if (r := item.upper()) and len(r) > 2)


# --- Complex expressions inside ---

ternary_gen = (x if x > 0 else -x for x in [-3, -1, 0, 2, 4])
list(ternary_gen)

lambda_gen = ((lambda n: n * 2)(x) for x in range(5))
list(lambda_gen)

tuple_gen = ((x, x + 1, x + 2) for x in range(5))
list(tuple_gen)

dict_inside = ({k: v} for k, v in [("a", 1), ("b", 2)])
list(dict_inside)

list_inside = ([x, x * 2] for x in range(5))
list(list_inside)

set_inside = ({x, x + 1} for x in range(5))
list(set_inside)

nested_call = (sorted(row) for row in [[3, 1, 2], [6, 4, 5]])
list(nested_call)

sliced = (s[::-1] for s in ["hello", "world"])
list(sliced)

starred_tuple = ((*pair,) for pair in [(1, 2), (3, 4)])
list(starred_tuple)


# --- Generator expression with unpacking ---

pair_gen = ((a, b) for a, b in [(1, 2), (3, 4), (5, 6)])
list(pair_gen)

triple_gen = ((a, b, c) for a, b, c in [(1, 2, 3), (4, 5, 6)])
list(triple_gen)

# NOTE: star in for-loop target inside genexpr not supported
# star_unpack = (first for first, *_ in [(1, 2, 3), (4, 5, 6)])
# list(star_unpack)

# NOTE: nested paren tuple in for target inside genexpr not supported
# nested_unpack = (a + d for (a, b), (c, d) in [((1, 2), (3, 4))])
# list(nested_unpack)


# --- Generator expression scope ---

x = 10
g = (x for x in range(5))
list(g)
x  # outer x unchanged

items = [1, 2, 3]
g = (items for items in [items])
list(g)


# --- Chaining generators ---

g1 = (x for x in range(5))
g2 = (x * 2 for x in g1)
g3 = (x + 1 for x in g2)
list(g3)


# --- Early termination patterns ---

from itertools import islice
first_five = islice((x * x for x in range(1000)), 5)
list(first_five)


# --- Boolean context ---

bool(x for x in range(0))  # generator is always truthy
any(x > 100 for x in range(10))
all(x < 100 for x in range(10))


# --- Comparison and identity ---

type(x for x in range(10))
(x for x in range(5)) is not (x for x in range(5))


# --- Empty iteration source ---

empty = (x for x in [])
list(empty)

empty_filtered = (x for x in range(10) if False)
list(empty_filtered)


# --- Genexp with attribute access and indexing ---

class Obj:
    def __init__(self, val):
        self.val = val

objs = [Obj(1), Obj(2), Obj(3)]
vals = (o.val for o in objs)
list(vals)

data = [[1, 2], [3, 4], [5, 6]]
firsts = (row[0] for row in data)
list(firsts)

lasts = (row[-1] for row in data)
list(lasts)

slices = (row[1:] for row in data)
list(slices)
