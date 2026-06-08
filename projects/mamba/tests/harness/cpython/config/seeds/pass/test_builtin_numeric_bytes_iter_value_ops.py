# Operational AssertionPass seed for the value contract of core
# Python built-in language operations: byte / bytearray byte-level
# helpers, numeric conversion builtins (int / float / hex / oct /
# bin / chr / ord / abs / round / divmod / pow), complex-number
# arithmetic, iter / next sentinel + default protocol, the f-string
# / format() format-spec mini-language, the type / isinstance /
# issubclass identity surface, and the comprehension family
# (list / dict / set / generator).
#
# The matching subset between mamba and CPython is the byte-
# indexing + byte-search + byte-mutation layer + integer-radix
# coercion + complex-product layer + iter-sentinel layer +
# format-spec mini-language layer + simple comprehension layer:
# b"Hello"[0] == 72, b"Hello".find(b"l") == 2, bytes.fromhex
# round-trip, bytearray append / extend / reverse / item-assign,
# int("ff", 16) == 255, abs(3+4j) == 5.0, round(3.14159, 2) ==
# 3.14, divmod(7, 3) == (2, 1), pow(2, 10, 1000) == 24, (1+2j) *
# (3+4j) == (-5+10j), iter([10,20]) + next default + StopIteration,
# f"{42:05d}" == "00042", f"{255:x}" == "ff", f"{'abc':*^10}" ==
# "***abc****", type(1).__name__ == "int", isinstance(True, int)
# is True, issubclass(bool, int) is True, [x for x in range(10)
# if x > 4], {k: v for k, v in pairs} dict comprehensions.
#
# Surface in this fixture:
#   • bytes — indexing, find / startswith / endswith / replace /
#     split / strip / hex / count, bytes.fromhex round-trip;
#   • bytearray — append / extend / reverse / item assignment;
#   • numeric — int / float / hex / oct / bin / chr / ord / abs /
#     round / divmod / pow conversions on canonical inputs;
#   • complex — complex(1, 2) constructor + .real / .imag /
#     .conjugate() + arithmetic;
#   • iter / next — list iter + StopIteration + default sentinel;
#   • format spec — int padding / hex / oct / bin, float width /
#     precision, string alignment / fill char, f-string !r / !s;
#   • type / isinstance / issubclass — built-in type identity +
#     bool-is-int subclassing + user-defined hierarchy;
#   • comprehensions — list / dict / set / generator + multi-for
#     + walrus + and-conjoined predicate;
#   • exception — args tuple + str(e) message round-trip.
#
# Behavioral edges that DIVERGE on mamba (bytes.upper / lower /
# title / swapcase / capitalize / isalpha / isdigit
# AttributeError, instance __class__.__name__ returning None,
# str.__add__ / tuple.__add__ / bytes.__add__ AttributeError,
# nested-if comprehensions silently dropping the second clause,
# built-in exception class __name__ returning None) are covered
# in the matching spec fixture `lang_bytes_class_comp_silent`.


class _ShapeBase:
    def area(self) -> int:
        return 0


class _Square(_ShapeBase):
    def area(self) -> int:
        return 4


_ledger: list[int] = []

# 1) bytes — byte-level indexing + search
_bs = b"Hello"
assert _bs[0] == 72; _ledger.append(1)
assert _bs.find(b"l") == 2; _ledger.append(1)
assert _bs.startswith(b"He") == True; _ledger.append(1)
assert _bs.endswith(b"lo") == True; _ledger.append(1)
assert _bs.replace(b"l", b"L") == b"HeLLo"; _ledger.append(1)
assert b"a,b,c".split(b",") == [b"a", b"b", b"c"]; _ledger.append(1)
assert b"  hi  ".strip() == b"hi"; _ledger.append(1)
assert _bs.hex() == "48656c6c6f"; _ledger.append(1)
assert _bs.count(b"l") == 2; _ledger.append(1)
assert bytes.fromhex("48656c6c6f") == b"Hello"; _ledger.append(1)

# 2) bytearray — mutation contract via bytearray-equality
_ba = bytearray(b"abc")
_ba.append(100)
assert _ba == bytearray(b"abcd"); _ledger.append(1)
_ba.extend(b"ef")
assert _ba == bytearray(b"abcdef"); _ledger.append(1)
_ba.reverse()
assert _ba == bytearray(b"fedcba"); _ledger.append(1)
_ba2 = bytearray(b"hello")
_ba2[0] = 72
assert _ba2 == bytearray(b"Hello"); _ledger.append(1)
assert _ba2[0] == 72; _ledger.append(1)
assert len(_ba2) == 5; _ledger.append(1)

# 3) numeric — radix conversions
assert int("42") == 42; _ledger.append(1)
assert int("ff", 16) == 255; _ledger.append(1)
assert int("101", 2) == 5; _ledger.append(1)
assert int(3.7) == 3; _ledger.append(1)
assert int(-3.7) == -3; _ledger.append(1)
assert float("1.5") == 1.5; _ledger.append(1)
assert hex(255) == "0xff"; _ledger.append(1)
assert oct(8) == "0o10"; _ledger.append(1)
assert bin(5) == "0b101"; _ledger.append(1)
assert chr(65) == "A"; _ledger.append(1)
assert ord("A") == 65; _ledger.append(1)
assert abs(-5) == 5; _ledger.append(1)
assert abs(-5.5) == 5.5; _ledger.append(1)
assert abs(3 + 4j) == 5.0; _ledger.append(1)
assert round(3.14159, 2) == 3.14; _ledger.append(1)
assert divmod(7, 3) == (2, 1); _ledger.append(1)
assert pow(2, 10) == 1024; _ledger.append(1)
assert pow(2, 10, 1000) == 24; _ledger.append(1)

# 4) complex — arithmetic + .real / .imag / .conjugate
_c = 3 + 4j
assert _c.real == 3.0; _ledger.append(1)
assert _c.imag == 4.0; _ledger.append(1)
assert _c.conjugate() == 3 - 4j; _ledger.append(1)
assert complex(1, 2) == 1 + 2j; _ledger.append(1)
assert (1 + 2j) + (3 + 4j) == 4 + 6j; _ledger.append(1)
assert (1 + 2j) * (3 + 4j) == -5 + 10j; _ledger.append(1)

# 5) iter / next — StopIteration + default sentinel
_it = iter([10, 20])
assert next(_it) == 10; _ledger.append(1)
assert next(_it) == 20; _ledger.append(1)
assert next(_it, "DONE") == "DONE"; _ledger.append(1)

# 6) format spec — int + float + string mini-language
assert f"{42:05d}" == "00042"; _ledger.append(1)
assert f"{3.14:.2f}" == "3.14"; _ledger.append(1)
assert f"{255:x}" == "ff"; _ledger.append(1)
assert f"{255:X}" == "FF"; _ledger.append(1)
assert f"{255:o}" == "377"; _ledger.append(1)
assert f"{255:b}" == "11111111"; _ledger.append(1)
assert f"{'abc':>10}" == "       abc"; _ledger.append(1)
assert f"{'abc':<10}" == "abc       "; _ledger.append(1)
assert f"{'abc':*^10}" == "***abc****"; _ledger.append(1)
assert format(42, "05d") == "00042"; _ledger.append(1)

# 7) type / isinstance / issubclass — built-in identity
assert type(1).__name__ == "int"; _ledger.append(1)
assert type("x").__name__ == "str"; _ledger.append(1)
assert type([]).__name__ == "list"; _ledger.append(1)
assert type({}).__name__ == "dict"; _ledger.append(1)
assert type(()).__name__ == "tuple"; _ledger.append(1)
assert type(None).__name__ == "NoneType"; _ledger.append(1)
assert isinstance(1, int) == True; _ledger.append(1)
assert isinstance(True, int) == True; _ledger.append(1)
assert isinstance(1.0, float) == True; _ledger.append(1)
assert isinstance(1, (int, str)) == True; _ledger.append(1)
assert issubclass(bool, int) == True; _ledger.append(1)
assert issubclass(int, object) == True; _ledger.append(1)
assert isinstance(_Square(), _ShapeBase) == True; _ledger.append(1)
assert issubclass(_Square, _ShapeBase) == True; _ledger.append(1)

# 8) comprehensions — list / dict / set / gen + multi-for + walrus
assert [x for x in range(10) if x > 4] == [5, 6, 7, 8, 9]; _ledger.append(1)
assert [x for x in range(10) if x % 2 == 0 and x > 4] == [6, 8]; _ledger.append(1)
assert {k: v for k, v in [("a", 1), ("b", 2)]} == {"a": 1, "b": 2}; _ledger.append(1)
assert sorted({x * 2 for x in range(5)}) == [0, 2, 4, 6, 8]; _ledger.append(1)
assert list(x ** 2 for x in range(5)) == [0, 1, 4, 9, 16]; _ledger.append(1)
assert [(x, y) for x in [1, 2] for y in ["a", "b"]] == [(1, "a"), (1, "b"), (2, "a"), (2, "b")]; _ledger.append(1)
assert [y for x in [1, 2, 3, 4] if (y := x * 2) > 3] == [4, 6, 8]; _ledger.append(1)

# 9) exception — args + str(e) round-trip
try:
    raise ValueError("test message")
except ValueError as _exc:
    assert str(_exc) == "test message"; _ledger.append(1)
    assert _exc.args == ("test message",); _ledger.append(1)

# NB: bytes.upper / lower / title / swapcase / capitalize /
# isalpha / isdigit AttributeError, instance __class__.__name__
# returning None, str.__add__ / tuple.__add__ / bytes.__add__
# AttributeError, nested-if comprehensions silently dropping the
# second clause, built-in exception class __name__ returning None
# all DIVERGE on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_builtin_numeric_bytes_iter_value_ops {sum(_ledger)} asserts")
