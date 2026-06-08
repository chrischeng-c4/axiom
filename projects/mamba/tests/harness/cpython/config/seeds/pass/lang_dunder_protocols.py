# Operational AssertionPass seed for dunder protocols not already
# exercised in lang_dunders.
# Surface: __iter__ wiring a class into the for-loop / list()
# protocol; __bool__ controlling truthiness in `bool()`, `if`, and
# `not`; __call__ making an instance callable + callable() probe;
# __le__ and __ge__ ordering operators alongside __lt__ / __gt__ /
# __eq__.
_ledger: list[int] = []


# __iter__ wires a class into the iterator protocol so `for x in obj`
# and `list(obj)` both work
class _Bag:
    def __init__(self, items):
        self.items = items
    def __iter__(self):
        return iter(self.items)

bg = _Bag([10, 20, 30])
# list(bg) drains __iter__ to a list
assert list(bg) == [10, 20, 30]; _ledger.append(1)
# for-loop also dispatches through __iter__
collected: list[int] = []
for v in bg:
    collected.append(v)
assert collected == [10, 20, 30]; _ledger.append(1)


# __bool__ controls truthiness — bool(obj) and `if obj` both consult it
class _Maybe:
    def __init__(self, has):
        self.has = has
    def __bool__(self):
        return self.has

# bool() returns True when __bool__ says yes
assert bool(_Maybe(True)) == True; _ledger.append(1)
# bool() returns False when __bool__ says no
assert bool(_Maybe(False)) == False; _ledger.append(1)
# `if obj:` consults __bool__ — the truthy branch runs
ran_truthy = 0
if _Maybe(True):
    ran_truthy = 1
assert ran_truthy == 1; _ledger.append(1)
# `if not obj:` is the inverse
ran_falsy = 0
if not _Maybe(False):
    ran_falsy = 1
assert ran_falsy == 1; _ledger.append(1)


# __call__ makes an instance callable — callable(obj) reports True
class _Adder:
    def __init__(self, base):
        self.base = base
    def __call__(self, x):
        return self.base + x

add5 = _Adder(5)
# Bind the int return to a local before subtraction-test to dodge
# int-identity-through-return (Task #15)
result = add5(3)
assert result - 8 == 0; _ledger.append(1)
# callable() probes the presence of __call__
assert callable(add5) == True; _ledger.append(1)


# Full ordering set: __le__ and __ge__ alongside __lt__ / __gt__
class _Rank:
    def __init__(self, n):
        self.n = n
    def __lt__(self, other):
        return self.n < other.n
    def __le__(self, other):
        return self.n <= other.n
    def __gt__(self, other):
        return self.n > other.n
    def __ge__(self, other):
        return self.n >= other.n
    def __eq__(self, other):
        return self.n == other.n

# Strict less-than
assert _Rank(1) < _Rank(2); _ledger.append(1)
# Equal-or-less via __le__
assert _Rank(1) <= _Rank(1); _ledger.append(1)
assert _Rank(1) <= _Rank(2); _ledger.append(1)
# Strict greater-than
assert _Rank(2) > _Rank(1); _ledger.append(1)
# Equal-or-greater via __ge__
assert _Rank(2) >= _Rank(2); _ledger.append(1)
assert _Rank(3) >= _Rank(1); _ledger.append(1)
# Equality via __eq__
assert _Rank(5) == _Rank(5); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_dunder_protocols {sum(_ledger)} asserts")
