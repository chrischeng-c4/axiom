# Operational AssertionPass seed for `nonlocal` and `global` name
# bindings.
# Surface: `nonlocal` binds an enclosing-function-scope name so the
# inner function's mutations are visible to the outer function;
# `global` binds the module-scope name so a function's `+=` mutates
# the module-scope variable. Return values are bound to locals before
# equality assertions to dodge the int-identity-through-return drop.
_ledger: list[int] = []

def make_counter():
    n = 0
    def inc():
        nonlocal n
        n += 1
        return n
    inc()
    inc()
    inc()
    return n

# After three inc() calls, the enclosing n is 3 (not 0)
c1 = make_counter()
assert c1 == 3; _ledger.append(1)

# A second invocation of make_counter starts from fresh state
c2 = make_counter()
assert c2 == 3; _ledger.append(1)

# Multi-level nonlocal: middle function binds outer's name; the
# innermost mutates the same slot
def three_level():
    x = 10
    def middle():
        def inner():
            nonlocal x
            x += 5
        inner()
        inner()
    middle()
    return x

# inner() ran twice, each adding 5 to the outer-most x (10+5+5=20)
t1 = three_level()
assert t1 == 20; _ledger.append(1)

counter_g = 0

def inc_global():
    global counter_g
    counter_g += 1
    return counter_g

# Two inc_global calls — counter_g progresses 0 → 1 → 2
a = inc_global()
b = inc_global()
assert a == 1; _ledger.append(1)
assert b == 2; _ledger.append(1)
assert counter_g == 2; _ledger.append(1)

# Reset global, then a fresh call ticks from the reset value
counter_g = 100
d = inc_global()
assert d == 101; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_nonlocal_global {sum(_ledger)} asserts")
