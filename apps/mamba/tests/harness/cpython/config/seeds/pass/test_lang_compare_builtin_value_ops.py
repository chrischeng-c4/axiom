# Atomic 321 pass conformance — language-core same-type ordering
# comparisons (int<int, float<float, str<str, list<list, tuple<tuple)
# plus cross-type equality (which returns False without raising in
# both runtimes) plus core built-in callables (len, abs, min, max,
# sum, sorted, reversed, any, all, enumerate, zip, map, filter,
# range, round, divmod, hex, oct, bin, chr, ord, repr, str, type,
# isinstance, issubclass). All asserts match between CPython 3.12
# and mamba.

_ledger: list[int] = []

# 1) same-type int/float ordering
assert 1 < 2; _ledger.append(1)
assert 2 > 1; _ledger.append(1)
assert 1 <= 1; _ledger.append(1)
assert 1 >= 1; _ledger.append(1)
assert 1.0 < 2.0; _ledger.append(1)
assert 2.0 > 1.0; _ledger.append(1)
assert 1 < 2.0; _ledger.append(1)
assert 2.0 > 1; _ledger.append(1)

# 2) same-type str ordering
assert "a" < "b"; _ledger.append(1)
assert "b" > "a"; _ledger.append(1)
assert "a" <= "a"; _ledger.append(1)
assert "ab" < "ac"; _ledger.append(1)

# 3) same-type list/tuple ordering
assert [1] < [2]; _ledger.append(1)
assert [1, 2] < [1, 3]; _ledger.append(1)
assert [] < [1]; _ledger.append(1)
assert (1,) < (2,); _ledger.append(1)
assert (1, 2) < (1, 3); _ledger.append(1)
assert () < (1,); _ledger.append(1)

# 4) cross-type equality (False without raising — both runtimes agree)
assert (1 == "a") == False; _ledger.append(1)
assert (1 == None) == False; _ledger.append(1)
assert ([] == ()) == False; _ledger.append(1)
assert ({} == []) == False; _ledger.append(1)
assert (None == None) == True; _ledger.append(1)
assert 1 == 1.0; _ledger.append(1)
assert (1 != "a") == True; _ledger.append(1)
assert (None != 0) == True; _ledger.append(1)

# 5) builtins — len
assert len([1, 2, 3]) == 3; _ledger.append(1)
assert len("abc") == 3; _ledger.append(1)
assert len({1, 2, 3}) == 3; _ledger.append(1)
assert len((1, 2)) == 2; _ledger.append(1)
assert len({"a": 1}) == 1; _ledger.append(1)
assert len(b"abc") == 3; _ledger.append(1)

# 6) builtins — abs
assert abs(-5) == 5; _ledger.append(1)
assert abs(5) == 5; _ledger.append(1)
assert abs(-5.5) == 5.5; _ledger.append(1)
assert abs(0) == 0; _ledger.append(1)

# 7) builtins — min/max/sum
assert min([3, 1, 2]) == 1; _ledger.append(1)
assert max([1, 3, 2]) == 3; _ledger.append(1)
assert min(3, 1, 2) == 1; _ledger.append(1)
assert max(1, 3, 2) == 3; _ledger.append(1)
assert sum([1, 2, 3]) == 6; _ledger.append(1)
assert sum([1, 2, 3], 10) == 16; _ledger.append(1)

# 8) builtins — sorted/reversed
assert sorted([3, 1, 2]) == [1, 2, 3]; _ledger.append(1)
assert sorted([3, 1, 2], reverse=True) == [3, 2, 1]; _ledger.append(1)
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)

# 9) builtins — any/all
assert any([0, 1, 0]) == True; _ledger.append(1)
assert any([0, 0]) == False; _ledger.append(1)
assert all([1, 2, 3]) == True; _ledger.append(1)
assert all([1, 0, 3]) == False; _ledger.append(1)

# 10) builtins — enumerate/zip/map/filter
assert list(enumerate(["a", "b"])) == [(0, "a"), (1, "b")]; _ledger.append(1)
assert list(zip([1, 2], [3, 4])) == [(1, 3), (2, 4)]; _ledger.append(1)
assert list(map(str, [1, 2])) == ["1", "2"]; _ledger.append(1)
assert list(filter(None, [0, 1, 2, 0, 3])) == [1, 2, 3]; _ledger.append(1)

# 11) builtins — range
assert list(range(3)) == [0, 1, 2]; _ledger.append(1)
assert list(range(1, 4)) == [1, 2, 3]; _ledger.append(1)
assert list(range(0, 10, 2)) == [0, 2, 4, 6, 8]; _ledger.append(1)

# 12) builtins — round/divmod
assert round(1.5) == 2; _ledger.append(1)
assert round(2.5) == 2; _ledger.append(1)
assert round(1.234, 2) == 1.23; _ledger.append(1)
assert divmod(7, 3) == (2, 1); _ledger.append(1)
assert divmod(10, 4) == (2, 2); _ledger.append(1)

# 13) builtins — hex/oct/bin/chr/ord
assert hex(255) == "0xff"; _ledger.append(1)
assert oct(8) == "0o10"; _ledger.append(1)
assert bin(5) == "0b101"; _ledger.append(1)
assert chr(65) == "A"; _ledger.append(1)
assert ord("A") == 65; _ledger.append(1)

# 14) builtins — repr/str on simple objects
assert str(1) == "1"; _ledger.append(1)
assert str(1.5) == "1.5"; _ledger.append(1)
assert str(True) == "True"; _ledger.append(1)
assert str(None) == "None"; _ledger.append(1)
assert repr("a") == "'a'"; _ledger.append(1)
assert repr(1) == "1"; _ledger.append(1)

# 15) builtins — isinstance/issubclass/type
assert isinstance(1, int) == True; _ledger.append(1)
assert isinstance("a", str) == True; _ledger.append(1)
assert isinstance([], list) == True; _ledger.append(1)
assert isinstance(1, str) == False; _ledger.append(1)
assert isinstance(True, int) == True; _ledger.append(1)
assert issubclass(bool, int) == True; _ledger.append(1)
assert issubclass(int, object) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_lang_compare_builtin_value_ops {sum(_ledger)} asserts")
