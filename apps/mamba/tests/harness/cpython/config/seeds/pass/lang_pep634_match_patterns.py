# Operational AssertionPass seed for PEP 634 — structural pattern
# matching (CPython 3.10+).
# Surface: literal patterns, OR patterns (`|`), capture patterns with
# guard (`case n if ...`), wildcard (`_`), class patterns with keyword
# subpatterns, sequence patterns (empty, fixed-arity, star), mapping
# patterns with key-value destructuring.
_ledger: list[int] = []

def classify(x):
    match x:
        case 0:
            return "zero"
        case 1 | 2 | 3:
            return "small"
        case n if n > 100:
            return "big"
        case _:
            return "other"

# Literal pattern
r1 = classify(0)
assert r1 == "zero"; _ledger.append(1)
# OR pattern — any of the three literals matches
r2 = classify(2)
assert r2 == "small"; _ledger.append(1)
# Capture pattern with guard
r3 = classify(200)
assert r3 == "big"; _ledger.append(1)
# Wildcard fallback
r4 = classify(50)
assert r4 == "other"; _ledger.append(1)

class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

def describe(p):
    match p:
        case Point(x=0, y=0):
            return "origin"
        case Point(x=0, y=y):
            return f"on-y:{y}"
        case Point(x=x, y=0):
            return f"on-x:{x}"
        case Point():
            return "elsewhere"
    return "none"

# Class pattern with literal keyword subpatterns
d1 = describe(Point(0, 0))
assert d1 == "origin"; _ledger.append(1)
# Class pattern with one literal + one capture
d2 = describe(Point(0, 5))
assert d2 == "on-y:5"; _ledger.append(1)
d3 = describe(Point(7, 0))
assert d3 == "on-x:7"; _ledger.append(1)
# Bare class pattern matches any instance
d4 = describe(Point(3, 4))
assert d4 == "elsewhere"; _ledger.append(1)

def head(seq):
    match seq:
        case []:
            return "empty"
        case [x]:
            return f"one:{x}"
        case [x, y]:
            return f"two:{x},{y}"
        case [x, *rest]:
            return f"many:{x},{len(rest)}"

# Empty sequence pattern
s1 = head([])
assert s1 == "empty"; _ledger.append(1)
# Fixed-arity sequence pattern
s2 = head([1])
assert s2 == "one:1"; _ledger.append(1)
s3 = head([1, 2])
assert s3 == "two:1,2"; _ledger.append(1)
# Star pattern captures the rest
s4 = head([1, 2, 3, 4])
assert s4 == "many:1,3"; _ledger.append(1)

def lookup(d):
    match d:
        case {"type": "ok", "value": v}:
            return f"ok:{v}"
        case {"type": "err"}:
            return "err"
        case _:
            return "?"

# Mapping pattern with literal value + capture
m1 = lookup({"type": "ok", "value": 42})
assert m1 == "ok:42"; _ledger.append(1)
# Mapping pattern matches subset of keys
m2 = lookup({"type": "err", "code": 500})
assert m2 == "err"; _ledger.append(1)
# No mapping case matches → wildcard
m3 = lookup({"other": "x"})
assert m3 == "?"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep634_match_patterns {sum(_ledger)} asserts")
