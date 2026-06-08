"""Behavior contract for builtins.format.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: format(value) — no format spec, same as str()
assert format(0) == "0", f"format(0) = {format(0)!r}"
assert format(42) == "42", f"format(42) = {format(42)!r}"
assert format(3.14) == "3.14", f"format(3.14) = {format(3.14)!r}"
assert format("hi") == "hi", f"format('hi') = {format('hi')!r}"
assert format(True) == "True", f"format(True) = {format(True)!r}"
assert format(None) == "None", f"format(None) = {format(None)!r}"

# Rule 2: integer format specs
assert format(42, "d") == "42", f"format(42,'d') = {format(42,'d')!r}"
assert format(42, "b") == "101010", f"format(42,'b') = {format(42,'b')!r}"
assert format(255, "x") == "ff", f"format(255,'x') = {format(255,'x')!r}"
assert format(255, "X") == "FF", f"format(255,'X') = {format(255,'X')!r}"
assert format(8, "o") == "10", f"format(8,'o') = {format(8,'o')!r}"
assert format(42, "#b") == "0b101010", f"format(42,'#b') = {format(42,'#b')!r}"
assert format(255, "#x") == "0xff", f"format(255,'#x') = {format(255,'#x')!r}"

# Rule 3: width and fill
assert format(42, "10d") == "        42", f"format(42,'10d') = {format(42,'10d')!r}"
assert format(42, "<10d") == "42        ", f"format(42,'<10d') = {format(42,'<10d')!r}"
assert format(42, ">10d") == "        42", f"format(42,'>10d') = {format(42,'>10d')!r}"
assert format(42, "^10d") == "    42    ", f"format(42,'^10d') = {format(42,'^10d')!r}"
assert format(42, "010d") == "0000000042", f"format(42,'010d') = {format(42,'010d')!r}"

# Rule 4: float format specs
assert format(3.14159, ".2f") == "3.14", f"format(3.14159,'.2f') = {format(3.14159,'.2f')!r}"
assert format(3.14159, ".4f") == "3.1416", f"format(3.14159,'.4f') = {format(3.14159,'.4f')!r}"
assert format(12345.6, ",.1f") == "12,345.6", f"format(12345.6,',.1f') = {format(12345.6,',.1f')!r}"
assert format(3.14e6, ".2e") == "3.14e+06", f"format(3.14e6,'.2e') = {format(3.14e6,'.2e')!r}"

# Rule 5: string format specs
assert format("hi", "10s") == "hi        ", f"format('hi','10s') = {format('hi','10s')!r}"
assert format("hi", ">10s") == "        hi", f"format('hi','>10s') = {format('hi','>10s')!r}"
assert format("hi", "^10s") == "    hi    ", f"format('hi','^10s') = {format('hi','^10s')!r}"

# Rule 6: format delegates to __format__
class _Custom:
    def __format__(self, spec: str) -> str:
        return f"custom:{spec}"
assert format(_Custom(), "abc") == "custom:abc", f"custom __format__ = {format(_Custom(), 'abc')!r}"

# Rule 7: empty spec delegates to __format__ with ""
class _Default:
    def __format__(self, spec: str) -> str:
        return f"default({spec!r})"
assert format(_Default()) == "default('')", f"default __format__ = {format(_Default())!r}"

print("behavior OK")
