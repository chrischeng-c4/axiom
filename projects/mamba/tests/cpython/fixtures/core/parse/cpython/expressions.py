# RUN: parse
# CPython-derived: expressions (literals, calls, attrs, indexing, slicing, ternary, walrus)

# --- integer literals ---
0
42
1000000

# --- float literals ---
3.14
0.0
1e10

# --- string literals ---
"hello"

# --- boolean and None ---
True
False
None

# --- ellipsis ---
...

# --- unary operators ---
-42
not True
~0xFF

# --- ternary expression ---
x = 1 if True else 0

# --- walrus operator ---
x = (n := 10)

# --- function call ---
print(42)
len("hello")

# --- call with keyword args ---
func(a=1, b=2)

# --- call with *args and **kwargs ---
func(*args, **kwargs)

# --- attribute access ---
obj.attr
obj.method(1, 2)

# --- chained attribute ---
a.b.c.d

# --- chained method + attr ---
obj.method().attr.other()

# --- indexing ---
items[0]
items[i]

# --- slicing ---
items[1:3]
items[:5]
items[2:]
items[::2]
items[1:10:3]

# --- nested indexing ---
matrix[0][1]

# --- tuple literal ---
(1, 2, 3)
()

# --- list literal ---
[1, 2, 3]
[]

# --- dict literal ---
{"a": 1, "b": 2}
{}

# --- set literal ---
{1, 2, 3}

# --- starred expression ---
(*rest,)

# --- Python-style lambda (untyped) ---
f = lambda x: x + 1
g = lambda x, y: x + y
h = lambda: 42

# --- Python-style lambda with defaults ---
f = lambda x, y=10: x + y

# --- Mamba-style lambda (typed) ---
f = lambda x: int: x + 1
