"""Behavior contract for builtins.bin, hex, and oct.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

# Rule 1: bin() correct values
assert bin(0) == "0b0", f"bin(0) = {bin(0)!r}"
assert bin(1) == "0b1", f"bin(1) = {bin(1)!r}"
assert bin(10) == "0b1010", f"bin(10) = {bin(10)!r}"
assert bin(255) == "0b11111111", f"bin(255) = {bin(255)!r}"
assert bin(-1) == "-0b1", f"bin(-1) = {bin(-1)!r}"
assert bin(-10) == "-0b1010", f"bin(-10) = {bin(-10)!r}"

# Rule 2: hex() correct values
assert hex(0) == "0x0", f"hex(0) = {hex(0)!r}"
assert hex(255) == "0xff", f"hex(255) = {hex(255)!r}"
assert hex(256) == "0x100", f"hex(256) = {hex(256)!r}"
assert hex(-1) == "-0x1", f"hex(-1) = {hex(-1)!r}"
assert hex(3735928559) == "0xdeadbeef", f"hex(deadbeef) = {hex(3735928559)!r}"

# Rule 3: oct() correct values
assert oct(0) == "0o0", f"oct(0) = {oct(0)!r}"
assert oct(8) == "0o10", f"oct(8) = {oct(8)!r}"
assert oct(255) == "0o377", f"oct(255) = {oct(255)!r}"
assert oct(-8) == "-0o10", f"oct(-8) = {oct(-8)!r}"

# Rule 4: bin/hex/oct on bool
assert bin(True) == "0b1", f"bin(True) = {bin(True)!r}"
assert hex(False) == "0x0", f"hex(False) = {hex(False)!r}"

# Rule 5: bin/hex/oct raise TypeError for float
for fn, name in [(bin, "bin"), (hex, "hex"), (oct, "oct")]:
    _raised = False
    try:
        fn(3.14)  # type: ignore[arg-type]
    except TypeError:
        _raised = True
    assert _raised, f"{name}(3.14) did not raise TypeError"

# Rule 6: bin/hex/oct support __index__
class _I:
    def __index__(self) -> int:
        return 10
assert bin(_I()) == "0b1010", f"bin(__index__) = {bin(_I())!r}"
assert hex(_I()) == "0xa", f"hex(__index__) = {hex(_I())!r}"
assert oct(_I()) == "0o12", f"oct(__index__) = {oct(_I())!r}"

# Rule 7: int("0b...", 2) round-trip
assert int(bin(42)[2:], 2) == 42, "bin round-trip"
assert int(hex(255)[2:], 16) == 255, "hex round-trip"
assert int(oct(64)[2:], 8) == 64, "oct round-trip"

print("behavior OK")
