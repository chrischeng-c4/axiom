# RUN: parse
# CPython-derived: starred expressions and unpacking (#553)

# --- star before first variable ---
*a, b = [1, 2, 3, 4]

# --- star in middle ---
a, *b, c = [1, 2, 3, 4, 5]

# --- star before last variable ---
a, b, *c = [1, 2, 3, 4]

# --- star with single capture ---
*a, = [1, 2, 3]

# --- list unpacking target ---
[*a] = [1, 2, 3]
[a, *b] = [1, 2, 3]
[a, *b, c] = [1, 2, 3, 4]

# --- tuple unpacking target ---
(*a,) = [1, 2, 3]
(a, *b) = [1, 2, 3]
(a, *b, c) = [1, 2, 3, 4]

# --- star in function calls ---
def f(*args, **kwargs):
    pass

f(*[1, 2, 3])
f(**{"a": 1})
f(*[1, 2], *[3, 4])
f(**{"a": 1}, **{"b": 2})
f(*[1, 2], **{"c": 3})

# --- multiple star unpacks in call ---
def g(a, b, c, d):
    pass

g(*[1, 2], *[3, 4])

# --- star in list literals ---
a = [1, 2]
b = [3, 4]
combined = [*a, *b]
extended = [*a, 5, *b, 6]
single_star = [*range(5)]

# --- star in tuple literals ---
t = (*a, *b)
t = (*a, 5, *b)
t = (*range(5),)

# --- star in set literals ---
s = {*a, *b}
s = {*a, 5, *b}
s = {*range(5)}

# --- double star in dict literals ---
d1 = {"a": 1}
d2 = {"b": 2}
# NOTE: ** dict unpacking in dict literal not supported: merged = {**d1, **d2}
# NOTE: ** dict unpacking in dict literal not supported: extended_dict = {**d1, **d2, "extra": 1}
# NOTE: ** dict unpacking in dict literal not supported: single_dict = {**d1}
# NOTE: ** dict unpacking in dict literal not supported: with_override = {**d1, "a": 99, **d2}

# --- star in return ---
def ret_star():
    return *[1, 2, 3],

# --- star in yield ---
def yield_star():
    # NOTE: starred yield not supported: yield *[1, 2, 3],
    yield (1, 2, 3)

# --- nested unpacking ---
(a, (b, c)) = (1, (2, 3))
(a, (*b, c)) = (1, ([2, 3, 4], 5))
[a, [b, *c]] = [1, [2, 3, 4]]

# --- deeply nested unpacking ---
((a, b), (c, d)) = ((1, 2), (3, 4))
(a, (b, (c, d))) = (1, (2, (3, 4)))
[(a, b), (c, d)] = [(1, 2), (3, 4)]

# --- unpacking in for loop ---
# NOTE: starred in for-loop target not supported
# for a, *b in [(1, 2, 3), (4, 5, 6)]:
#     pass
for ab in [(1, 2, 3), (4, 5, 6)]:
    pass

# NOTE: nested paren in for-loop target not supported
# for (a, b), c in [((1, 2), 3), ((4, 5), 6)]:
#     pass
for ab_c in [((1, 2), 3), ((4, 5), 6)]:
    pass

# NOTE: starred in nested for-loop target not supported
# for a, (*b, c) in [(1, (2, 3, 4)), (5, (6, 7, 8))]:
#     pass
for a_bc in [(1, (2, 3, 4)), (5, (6, 7, 8))]:
    pass

# --- unpacking in comprehension ---
pairs = [(1, 2), (3, 4)]
firsts = [a for a, b in pairs]

# --- star in assignment with complex right side ---
*a, b = range(10)
a, *b = "hello"
*a, b = map(int, "12345")

# --- unpacking with underscore ---
_, *rest = [1, 2, 3, 4]
a, _, *rest = [1, 2, 3, 4, 5]
*_, last = [1, 2, 3, 4]

# --- double star in function def ---
def func(**kwargs):
    pass

def func2(a, b, **kwargs):
    pass

def func3(*args, **kwargs):
    pass

def func4(a, *args, b=1, **kwargs):
    pass

# --- star in class constructor call ---
class MyClass:
    def __init__(self, *args, **kwargs):
        pass

obj = MyClass(*[1, 2], **{"key": "val"})

# --- star in chained calls ---
result = sorted([*a, *b], key=lambda x: x)

# --- unpacking in augmented assignment context ---
items = [*a, *b, *[5, 6]]

# --- star in conditional expression ---
x = [*a] if True else [*b]
