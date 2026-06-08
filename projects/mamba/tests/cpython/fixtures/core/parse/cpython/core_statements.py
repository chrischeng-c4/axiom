# RUN: parse
# CPython-derived: basic statements (assign, augmented assign, del, global, nonlocal, assert, raise)

# --- Variable declarations with type annotations ---
x: int = 42
y: float = 3.14
name: str = "hello"
flag: bool = True
nothing: int? = None

# --- Plain assignment ---
x = 100
x = y

# --- Augmented assignment (all operators) ---
x += 1
x -= 2
x *= 3
x /= 4
x //= 5
x %= 6
x **= 7
x &= 8
x |= 9
x ^= 10
x <<= 1
x >>= 2

# --- pass, break, continue (inside control flow) ---
while True:
    pass
while True:
    break
while True:
    continue

# --- del ---
del x

# --- global / nonlocal ---
def outer() -> int:
    global x
    def inner() -> int:
        nonlocal x
        return 0
    return 0

# --- assert ---
assert True
assert x == 42, "x must be 42"

# --- raise ---
raise ValueError("bad")
raise TypeError("wrong") from ValueError("cause")
def reraise() -> int:
    raise
    return 0
