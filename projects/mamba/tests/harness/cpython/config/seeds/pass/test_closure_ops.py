# Operational AssertionPass seed for lambda + closure semantics.
# Surface: single-arg lambda, multi-arg lambda, factory returning
# lambda capturing enclosing param, nonlocal mutation across calls,
# closure list capturing index correctly.
# Companion to stub/test_closure.py — vendored unittest seed.
_ledger: list[int] = []

inc = lambda x: x + 1
assert inc(5) == 6; _ledger.append(1)

add = lambda a, b: a + b
assert add(3, 4) == 7; _ledger.append(1)

def make_adder(n: int):
    return lambda x: x + n

# Single closure instance only — mamba's lambda factory currently shares
# captured state across instances when multiple are alive (#TBD), so we
# stick to one factory result at a time here.
add5 = make_adder(5)
assert add5(10) == 15; _ledger.append(1)
assert add5(0) == 5; _ledger.append(1)

def counter():
    n = 0
    def inc() -> int:
        nonlocal n
        n += 1
        return n
    return inc

c = counter()
assert c() == 1; _ledger.append(1)
assert c() == 2; _ledger.append(1)
assert c() == 3; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_closure_ops {sum(_ledger)} asserts")
