# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Tuple-unpack assignments inside generator bodies (and other function
# bodies) silently vanished from the lowered HIR. The classic Fibonacci
# generator produced `[None, None, ...]` instead of the Fibonacci
# sequence:
#
#   def fib(n):
#       a, b = 0, 1
#       for _ in range(n):
#           yield a
#           a, b = b, a + b
#
# Root cause (`lower/ast_to_hir.rs::lower_lvalue`): the `Ident` arm
# resolved the name and propagated `None` via `?` when the name was
# unbound. For a single-target assignment the surrounding `Stmt::Assign`
# arm pre-defines the local, but for a tuple/unpack target the
# statement falls through to `lower_lvalue(target)?`, so when `a` (or
# any sub-target of the tuple) was being defined for the first time,
# the entire `Assign` returned `None` and was silently dropped from
# the HIR.
#
# Fix: in `lower_lvalue`'s `Ident` arm, fall back to `define_local`
# (with `Any` type) when the name is unbound — the same pattern the
# single-ident `Stmt::Assign` path already uses. Tuple-unpack targets
# now lower to a real `HirStmt::Assign`, and the values land in the
# right vregs.
#
# This bug was hiding behind the much rarer case where the LHS names
# were *already* bound in an outer scope or earlier in the function;
# anything truly first-defined via tuple-unpack vanished.

# Classic Fibonacci generator — the canonical motivating case.
def fib(n):
    a, b = 0, 1
    for _ in range(n):
        yield a
        a, b = b, a + b

print(list(fib(10)))    # [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
print(list(fib(0)))     # []
print(list(fib(1)))     # [0]

# First-time tuple unpack inside a generator body — the minimal repro.
def gA():
    a, b = 0, 1
    yield a
    yield b
print(list(gA()))       # [0, 1]

# Tuple-unpack from a parenthesised RHS.
def gH():
    a, b = (0, 1)
    yield a
    yield b
print(list(gH()))       # [0, 1]

# Tuple-unpack from a variable.
def gI():
    pair = (0, 1)
    a, b = pair
    yield (a, b)
print(list(gI()))       # [(0, 1)]

# Three-target unpack from a list.
def gJ():
    a, b, c = [10, 20, 30]
    yield a
    yield b
    yield c
print(list(gJ()))       # [10, 20, 30]

# Same pattern outside a generator — must keep working.
def regular():
    a, b = 5, 7
    return (a, b)
print(regular())        # (5, 7)

# Nested tuple unpack inside a regular function (no yield).
def nested():
    pair = (1, 2)
    a, b = pair
    c, d = b, a
    return (a, b, c, d)
print(nested())         # (1, 2, 2, 1)
