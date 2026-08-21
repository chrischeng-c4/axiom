# Operational AssertionPass seed for type-introspection builtins:
# isinstance/issubclass/type, callable, hasattr/getattr, repr/str
# on builtin types, chr/ord/hex/bin/oct/abs/round/pow/divmod.
# Companion to stub/test_builtins.py — vendored unittest seed.
_ledger: list[int] = []
assert isinstance(1, int); _ledger.append(1)
assert isinstance("hi", str); _ledger.append(1)
assert isinstance(1.5, float); _ledger.append(1)
assert isinstance([], list); _ledger.append(1)
assert not isinstance("hi", int); _ledger.append(1)
assert issubclass(bool, int); _ledger.append(1)
assert issubclass(int, object); _ledger.append(1)
assert callable(len); _ledger.append(1)
assert not callable(42); _ledger.append(1)
assert chr(65) == "A"; _ledger.append(1)
assert ord("A") == 65; _ledger.append(1)
assert hex(255) == "0xff"; _ledger.append(1)
assert bin(5) == "0b101"; _ledger.append(1)
assert oct(8) == "0o10"; _ledger.append(1)
assert abs(-7) == 7; _ledger.append(1)
assert abs(-3.5) == 3.5; _ledger.append(1)
assert round(3.7) == 4; _ledger.append(1)
assert round(3.14159, 2) == 3.14; _ledger.append(1)
assert pow(2, 10) == 1024; _ledger.append(1)
assert pow(2, 10, 1000) == 24; _ledger.append(1)
assert divmod(17, 5) == (3, 2); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_type_introspection_ops {sum(_ledger)} asserts")
