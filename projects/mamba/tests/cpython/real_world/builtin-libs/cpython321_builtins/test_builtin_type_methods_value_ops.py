# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_builtin_type_methods_value_ops"
# subject = "cpython321.test_builtin_type_methods_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_builtin_type_methods_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_builtin_type_methods_value_ops: execute CPython 3.12 seed test_builtin_type_methods_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 232 pass conformance — builtin type method value ops
# (str / list / dict / set / frozenset / int / float / range / tuple /
# slice basic / bytes conforming subset) that match between CPython 3.12
# and mamba.
_ledger: list[int] = []

# 1) str — full method surface
assert "hello world".upper() == "HELLO WORLD"; _ledger.append(1)
assert "HELLO".lower() == "hello"; _ledger.append(1)
assert "hello world".title() == "Hello World"; _ledger.append(1)
assert "hello world".capitalize() == "Hello world"; _ledger.append(1)
assert "Hello".swapcase() == "hELLO"; _ledger.append(1)
assert "hello world".split() == ["hello", "world"]; _ledger.append(1)
assert "hello world".split("o") == ["hell", " w", "rld"]; _ledger.append(1)
assert "hello world".rsplit("o", 1) == ["hello w", "rld"]; _ledger.append(1)
assert "-".join(["a", "b", "c"]) == "a-b-c"; _ledger.append(1)
assert "hello".replace("l", "L") == "heLLo"; _ledger.append(1)
assert "hello".count("l") == 2; _ledger.append(1)
assert "hello".find("ll") == 2; _ledger.append(1)
assert "hello".rfind("l") == 3; _ledger.append(1)
assert "hello".index("ll") == 2; _ledger.append(1)
assert "hello".startswith("he") == True; _ledger.append(1)
assert "hello".endswith("lo") == True; _ledger.append(1)
assert " hello ".strip() == "hello"; _ledger.append(1)
assert " hello ".lstrip() == "hello "; _ledger.append(1)
assert " hello ".rstrip() == " hello"; _ledger.append(1)
assert "abc".isalpha() == True; _ledger.append(1)
assert "123".isdigit() == True; _ledger.append(1)
assert "abc123".isalnum() == True; _ledger.append(1)
assert "   ".isspace() == True; _ledger.append(1)
assert "ABC".isupper() == True; _ledger.append(1)
assert "abc".islower() == True; _ledger.append(1)
assert "Hello World".istitle() == True; _ledger.append(1)
assert "123".isnumeric() == True; _ledger.append(1)
assert "123".isdecimal() == True; _ledger.append(1)
assert "foo".isidentifier() == True; _ledger.append(1)
assert "1foo".isidentifier() == False; _ledger.append(1)
assert "abc".isprintable() == True; _ledger.append(1)
assert "abc".isascii() == True; _ledger.append(1)
assert "42".zfill(5) == "00042"; _ledger.append(1)
assert "abc".ljust(7, "-") == "abc----"; _ledger.append(1)
assert "abc".rjust(7, "-") == "----abc"; _ledger.append(1)
assert "abc".center(9, "-") == "---abc---"; _ledger.append(1)
assert "hello world".partition("w") == ("hello ", "w", "orld"); _ledger.append(1)
assert "hello world".rpartition("o") == ("hello w", "o", "rld"); _ledger.append(1)
assert "HELLO".casefold() == "hello"; _ledger.append(1)
assert "hello world".removeprefix("hello ") == "world"; _ledger.append(1)
assert "hello world".removesuffix(" world") == "hello"; _ledger.append(1)
assert "abc".encode("utf-8") == b"abc"; _ledger.append(1)
assert "abc".encode() == b"abc"; _ledger.append(1)
assert str(42) == "42"; _ledger.append(1)
assert str([1, 2, 3]) == "[1, 2, 3]"; _ledger.append(1)
assert repr("hello") == "'hello'"; _ledger.append(1)
assert "".isalpha() == False; _ledger.append(1)

# 2) list — full method surface + slicing + concat/repeat
_xs = [3, 1, 4, 1, 5, 9, 2, 6]
_x_sort = list(_xs); _x_sort.sort()
assert _x_sort == [1, 1, 2, 3, 4, 5, 6, 9]; _ledger.append(1)
assert sorted(_xs) == [1, 1, 2, 3, 4, 5, 6, 9]; _ledger.append(1)
assert sorted(_xs, reverse=True) == [9, 6, 5, 4, 3, 2, 1, 1]; _ledger.append(1)
assert list(reversed(_xs)) == [6, 2, 9, 5, 1, 4, 1, 3]; _ledger.append(1)
_x_rev = list(_xs); _x_rev.reverse()
assert _x_rev == [6, 2, 9, 5, 1, 4, 1, 3]; _ledger.append(1)
_x_app = [1, 2, 3]; _x_app.append(99)
assert _x_app == [1, 2, 3, 99]; _ledger.append(1)
_x_ins = [1, 2, 3]; _x_ins.insert(1, 99)
assert _x_ins == [1, 99, 2, 3]; _ledger.append(1)
_x_ext = [1, 2]; _x_ext.extend([3, 4])
assert _x_ext == [1, 2, 3, 4]; _ledger.append(1)
_x_pop = [1, 2, 3]; _v_pop = _x_pop.pop()
assert _v_pop == 3; _ledger.append(1)
assert _x_pop == [1, 2]; _ledger.append(1)
_x_rm = [1, 2, 3, 2]; _x_rm.remove(2)
assert _x_rm == [1, 3, 2]; _ledger.append(1)
assert _xs.index(5) == 4; _ledger.append(1)
assert _xs.count(1) == 2; _ledger.append(1)
_x_clr = [1, 2, 3]; _x_clr.clear()
assert _x_clr == []; _ledger.append(1)
_x_cp = _xs.copy()
assert _x_cp == _xs; _ledger.append(1)
assert [1, 2, 3] + [4, 5] == [1, 2, 3, 4, 5]; _ledger.append(1)
assert [1, 2] * 3 == [1, 2, 1, 2, 1, 2]; _ledger.append(1)
assert [1] * 0 == []; _ledger.append(1)
assert list("abc") == ["a", "b", "c"]; _ledger.append(1)
assert list((1, 2, 3)) == [1, 2, 3]; _ledger.append(1)

# 3) dict — method surface (classmethod fromkeys / | merge / iter ctor)
_d = {"a": 1, "b": 2, "c": 3}
assert sorted(_d.keys()) == ["a", "b", "c"]; _ledger.append(1)
assert sorted(_d.values()) == [1, 2, 3]; _ledger.append(1)
assert sorted(_d.items()) == [("a", 1), ("b", 2), ("c", 3)]; _ledger.append(1)
assert _d.get("a") == 1; _ledger.append(1)
assert _d.get("z", "def") == "def"; _ledger.append(1)
_d_sd = _d.copy()
assert _d_sd.setdefault("z", 99) == 99; _ledger.append(1)
_d_up = _d.copy(); _d_up.update({"d": 4})
assert sorted(_d_up.items()) == [("a", 1), ("b", 2), ("c", 3), ("d", 4)]; _ledger.append(1)
_d_pop = _d.copy(); _v_dp = _d_pop.pop("a")
assert _v_dp == 1; _ledger.append(1)
_d_clr = _d.copy(); _d_clr.clear()
assert _d_clr == {}; _ledger.append(1)
assert dict.fromkeys(["a", "b"], 0) == {"a": 0, "b": 0}; _ledger.append(1)
assert ({"a": 1} | {"b": 2}) == {"a": 1, "b": 2}; _ledger.append(1)
assert dict() == {}; _ledger.append(1)
assert dict(a=1, b=2) == {"a": 1, "b": 2}; _ledger.append(1)
assert dict([("a", 1), ("b", 2)]) == {"a": 1, "b": 2}; _ledger.append(1)

# 4) set / frozenset — value ops
_s1 = {1, 2, 3, 4}
_s2 = {3, 4, 5, 6}
assert sorted(_s1 | _s2) == [1, 2, 3, 4, 5, 6]; _ledger.append(1)
assert sorted(_s1 & _s2) == [3, 4]; _ledger.append(1)
assert sorted(_s1 - _s2) == [1, 2]; _ledger.append(1)
assert sorted(_s1 ^ _s2) == [1, 2, 5, 6]; _ledger.append(1)
assert {1, 2}.issubset(_s1) == True; _ledger.append(1)
assert _s1.issuperset({1, 2}) == True; _ledger.append(1)
assert _s1.isdisjoint({99}) == True; _ledger.append(1)
_s_add = _s1.copy(); _s_add.add(99)
assert sorted(_s_add) == [1, 2, 3, 4, 99]; _ledger.append(1)
_s_dsc = _s1.copy(); _s_dsc.discard(1)
assert sorted(_s_dsc) == [2, 3, 4]; _ledger.append(1)
_s_rm = _s1.copy(); _s_rm.remove(1)
assert sorted(_s_rm) == [2, 3, 4]; _ledger.append(1)
assert len(_s1) == 4; _ledger.append(1)
assert (3 in _s1) == True; _ledger.append(1)
assert set([1, 2, 3]) == {1, 2, 3}; _ledger.append(1)
assert sorted(frozenset([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert sorted(frozenset([1, 2, 3]) | {4, 5}) == [1, 2, 3, 4, 5]; _ledger.append(1)
assert set().union({1, 2}) == {1, 2}; _ledger.append(1)

# 5) int — method surface + base ctors + numeric attrs (real/imag/conjugate)
assert (10).bit_length() == 4; _ledger.append(1)
assert (10).bit_count() == 2; _ledger.append(1)
assert (0).bit_length() == 0; _ledger.append(1)
assert (0xff).bit_count() == 8; _ledger.append(1)
assert (255).to_bytes(2, "big") == b"\x00\xff"; _ledger.append(1)
assert (255).to_bytes(4, "little") == b"\xff\x00\x00\x00"; _ledger.append(1)
assert int.from_bytes(b"\x00\xff", "big") == 255; _ledger.append(1)
assert int.from_bytes(b"\x00\xff", "little") == 65280; _ledger.append(1)
assert int("1010", 2) == 10; _ledger.append(1)
assert int("ff", 16) == 255; _ledger.append(1)
assert bin(10) == "0b1010"; _ledger.append(1)
assert hex(255) == "0xff"; _ledger.append(1)
assert oct(8) == "0o10"; _ledger.append(1)
assert abs(-5) == 5; _ledger.append(1)
assert divmod(10, 3) == (3, 1); _ledger.append(1)
assert round(2.5) == 2; _ledger.append(1)
assert round(3.5) == 4; _ledger.append(1)
assert round(2.6) == 3; _ledger.append(1)
assert int(3.7) == 3; _ledger.append(1)
assert (-5).__abs__() == 5; _ledger.append(1)

# 6) float — as_integer_ratio + is_integer + repr
assert (3.5).as_integer_ratio() == (7, 2); _ledger.append(1)
assert (2.5).as_integer_ratio() == (5, 2); _ledger.append(1)
assert (0.0).as_integer_ratio() == (0, 1); _ledger.append(1)
assert (5.0).is_integer() == True; _ledger.append(1)
assert (5.5).is_integer() == False; _ledger.append(1)
assert (0.1).is_integer() == False; _ledger.append(1)
assert (-0.0).is_integer() == True; _ledger.append(1)
assert repr(3.5) == "3.5"; _ledger.append(1)

# 7) range — value ops + slicing
assert list(range(5)) == [0, 1, 2, 3, 4]; _ledger.append(1)
assert list(range(2, 8)) == [2, 3, 4, 5, 6, 7]; _ledger.append(1)
assert list(range(0, 10, 2)) == [0, 2, 4, 6, 8]; _ledger.append(1)
assert len(range(100)) == 100; _ledger.append(1)
assert (3 in range(5)) == True; _ledger.append(1)
assert range(5)[2] == 2; _ledger.append(1)
assert list(range(10)[1:5]) == [1, 2, 3, 4]; _ledger.append(1)

# 8) tuple — value ops
_t = (1, 2, 3, 1, 2)
assert _t.count(1) == 2; _ledger.append(1)
assert _t.index(2) == 1; _ledger.append(1)
assert len(_t) == 5; _ledger.append(1)

# 9) basic list / string slicing
assert [1, 2, 3, 4, 5][1:3] == [2, 3]; _ledger.append(1)
assert [1, 2, 3, 4, 5][::2] == [1, 3, 5]; _ledger.append(1)
assert [1, 2, 3, 4, 5][::-1] == [5, 4, 3, 2, 1]; _ledger.append(1)
assert "hello"[1:3] == "el"; _ledger.append(1)
assert "hello"[::-1] == "olleh"; _ledger.append(1)

# 10) bytes — conforming subset
_b = b"hello world"
assert _b.split() == [b"hello", b"world"]; _ledger.append(1)
assert _b.replace(b"l", b"L") == b"heLLo worLd"; _ledger.append(1)
assert _b.hex() == "68656c6c6f20776f726c64"; _ledger.append(1)
assert bytes.fromhex("68656c6c6f") == b"hello"; _ledger.append(1)
assert _b.count(b"l") == 3; _ledger.append(1)
assert _b.find(b"world") == 6; _ledger.append(1)
assert _b.rfind(b"l") == 9; _ledger.append(1)
assert b" x ".strip() == b"x"; _ledger.append(1)
assert _b.decode() == "hello world"; _ledger.append(1)
assert _b.startswith(b"hello") == True; _ledger.append(1)
assert _b.endswith(b"world") == True; _ledger.append(1)
assert len(_b) == 11; _ledger.append(1)
assert b"-".join([b"a", b"b"]) == b"a-b"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_builtin_type_methods_value_ops {sum(_ledger)} asserts")
