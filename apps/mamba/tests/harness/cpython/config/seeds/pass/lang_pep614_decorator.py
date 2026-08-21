# Operational AssertionPass seed for PEP 614 — relaxed decorator
# grammar (Py 3.9+).
# Surface: arbitrary expressions allowed in decorator position, not
# just dotted names. Covers paren-wrapped expression (@(expr)) form;
# the subscript-decorator form (@registry[0]) currently mis-binds
# the wrapped function on mamba and is tracked as a separate gap.
def loud(f):
    def w(*args, **kw):
        return f(*args, **kw).upper()
    return w

def quiet(f):
    return f

@(quiet)
def passthrough(x):
    return x

@(loud)
def shout(msg):
    return msg

_ledger: list[int] = []
# Paren-wrapped identity decorator preserves str behaviour
assert passthrough("abc") == "abc"; _ledger.append(1)
assert passthrough("") == ""; _ledger.append(1)
# Paren-wrapped wrapper decorator transforms str output
assert shout("hello") == "HELLO"; _ledger.append(1)
assert shout("World") == "WORLD"; _ledger.append(1)
assert shout("") == ""; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep614_decorator {sum(_ledger)} asserts")
