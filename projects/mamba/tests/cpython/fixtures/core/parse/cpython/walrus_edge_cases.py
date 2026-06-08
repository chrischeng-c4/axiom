# RUN: parse
# CPython-derived: walrus operator (:=) edge cases (#550)

# --- walrus in while loop ---
def read_loop():
    data = None
    while (data := input()):
        process(data)

# --- walrus in if condition ---
def match_line(pattern, line):
    if (m := pattern.match(line)):
        return m.group(0)
    return None

# --- walrus in list comprehension ---
results = [y := f(x), y ** 2, y ** 3]

# --- nested walrus ---
val = (a := (b := 1))

# --- walrus in ternary ---
def ternary_walrus(f):
    x = "yes" if (y := f()) else "no"
    return x

# --- walrus in assert ---
def check_compute(compute):
    assert (x := compute()) > 0
    return x

# --- walrus with boolean operators ---
def bool_walrus(a, b):
    if (x := a) or (y := b):
        pass

# --- walrus with and operator ---
def and_walrus(a, b):
    if (x := a) and (y := b):
        pass

# --- walrus in function call args ---
def call_walrus():
    f(x := 10)
    g(y := 20, z := 30)

# --- walrus in dict comprehension values ---
data = [1, 2, 3, 4, 5]
result = {k: (v := k * 2) for k in data}

# --- walrus in set comprehension ---
seen = {(y := x * 2) for x in range(5)}

# --- walrus in generator expression ---
gen = ((y := x + 1) for x in range(5))

# --- multiple walrus in same expression ---
def multi_walrus():
    if (a := 1) + (b := 2) + (c := 3):
        return a + b + c

# --- walrus in conditional list building ---
filtered = [y for x in range(10) if (y := x * 2) > 5]

# --- walrus in while with complex expression ---
def complex_while():
    while (chunk := read(4096)) != b"":
        process(chunk)

# --- walrus in nested if ---
def nested_if():
    if (x := compute()):
        if (y := transform(x)):
            return y
    return None

# --- walrus assigned to tuple element ---
pair = ((a := 1), (b := 2))

# --- walrus in f-string ---
s = f"{(n := 42)} is the answer, and {n} again"

# --- walrus in subscript context ---
def subscript_walrus(data):
    result = data[(idx := find_index())]
    return result

# --- walrus in comparison chain ---
def chained(compute):
    if 0 < (x := compute()) < 100:
        return x
