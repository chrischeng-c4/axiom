# lang_pep695_generics.py — #3341 axis-1 PEP 695 generic class/function seed.
#
# Exercises the PEP 695 generic surface mamba services today:
#   1. `def f[T](...) -> T:` syntax — generic function definition + runtime
#      dispatch with int and str type-parameter instantiations
#   2. `class Box[T]:` syntax — generic class definition + construction
#      without explicit subscript
#   3. Generic class with multiple type parameters used independently across
#      instances
#   4. Generic function with multiple type parameters
#   5. Generic function returning a value derived from a type parameter
#
# Mamba quirks (tracked separately):
#   * Box[int]() (subscripted-class construction) returns None / raises
#     TypeError — covered in companion spec/lang_pep695_generics_spec.py
#   * Generic PEP 695 type-alias syntax (`type ListOrSet[T] = list[T] | set[T]`)
#     fails — tracked under #3485
#   * Float values reinterpreted as int bits inside tuple containers (e.g.
#     ('x', 3.14) becomes ('x', 4614253070214989087) — IEEE 754 raw bits);
#     this seed uses (int, str) and (str, str) tuples to dodge that quirk
#
# Contract (encoded by parent directory `pass/`): AssertionError → Fail;
# MAMBA_ASSERTION_PASS → AssertionPass.

_ledger: list[int] = []

# (1) Generic function with int type parameter
def _first[T](items: list[T]) -> T:
    return items[0]

assert _first([10, 20, 30]) - 10 == 0, (
    f"_first([10,20,30]) returns 10, got {_first([10,20,30])!r}"
)
_ledger.append(1)

# (2) Same generic function instantiated with str (same code path)
assert _first(["a", "b", "c"]) == "a", (
    f"_first(['a','b','c']) returns 'a', got {_first(['a','b','c'])!r}"
)
_ledger.append(1)

# (3) Generic function with two type parameters: pair builder
def _pair[A, B](a: A, b: B) -> tuple[A, B]:
    return (a, b)

_p = _pair(1, "hi")
assert _p == (1, "hi"), f"_pair(1, 'hi') == (1, 'hi'), got {_p!r}"
_ledger.append(1)

_p2 = _pair("x", "y")
assert _p2 == ("x", "y"), f"_pair('x', 'y') == ('x', 'y'), got {_p2!r}"
_ledger.append(1)

# (4) Generic class without subscript: plain construction
class _Box[T]:
    def __init__(self, v):
        self.v = v
    def get(self):
        return self.v

_b1 = _Box(42)
assert _b1.v - 42 == 0, f"_Box(42).v == 42, got {_b1.v!r}"
_ledger.append(1)

assert _b1.get() - 42 == 0, f"_Box(42).get() == 42, got {_b1.get()!r}"
_ledger.append(1)

# (5) Generic class with str argument
_b2 = _Box("hello")
assert _b2.v == "hello", f"_Box('hello').v == 'hello', got {_b2.v!r}"
_ledger.append(1)

# (6) Two instances of the same generic class don't share state
_a = _Box(1)
_b = _Box(2)
assert _a.v - 1 == 0 and _b.v - 2 == 0, (
    f"distinct _Box instances hold their own values, "
    f"_a.v={_a.v!r} _b.v={_b.v!r}"
)
_ledger.append(1)

# (7) Generic class with method that mutates instance state
class _Counter[T]:
    def __init__(self):
        self.value: int = 0
    def add(self, n):
        self.value = self.value + n

_c = _Counter()
_c.add(3)
_c.add(4)
assert _c.value - 7 == 0, (
    f"_Counter()+3+4 == 7, got {_c.value!r}"
)
_ledger.append(1)

# (8) Generic function returning a value from a list element
def _last[T](items: list[T]) -> T:
    return items[-1]

assert _last([1, 2, 3]) - 3 == 0, (
    f"_last([1,2,3]) returns 3, got {_last([1,2,3])!r}"
)
_ledger.append(1)

# (9) Generic class with two type parameters
class _KV[K, V]:
    def __init__(self, key, value):
        self.key = key
        self.value = value

_kv = _KV("name", 42)
assert _kv.key == "name" and _kv.value - 42 == 0, (
    f"_KV('name', 42) holds both, got key={_kv.key!r} value={_kv.value!r}"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pep695_generics {sum(_ledger)} asserts")
