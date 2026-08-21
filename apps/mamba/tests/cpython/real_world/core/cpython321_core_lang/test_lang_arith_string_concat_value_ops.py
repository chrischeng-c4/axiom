# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "test_lang_arith_string_concat_value_ops"
# subject = "cpython321.test_lang_arith_string_concat_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_lang_arith_string_concat_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_lang_arith_string_concat_value_ops: execute CPython 3.12 seed test_lang_arith_string_concat_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 320 pass conformance — language-core same-type and numeric-
# promoted arithmetic plus same-type sequence concatenation (int+float
# promotion, bool+int promotion, int*float promotion, str+str, str*int
# repeat, list+list, tuple+tuple, list*int repeat, set ops, dict |,
# string methods, unary +/-/~ on numerics). All asserts match between
# CPython 3.12 and mamba.

_ledger: list[int] = []

# 1) int <-> float promotion
assert 1 + 1.0 == 2.0; _ledger.append(1)
assert 1.0 + 1 == 2.0; _ledger.append(1)
assert 1 * 1.5 == 1.5; _ledger.append(1)
assert 2 ** 0.5 == 1.4142135623730951; _ledger.append(1)
assert 3 / 2 == 1.5; _ledger.append(1)
assert 3 // 2 == 1; _ledger.append(1)
assert 1 == 1.0; _ledger.append(1)

# 2) bool <-> int promotion
assert 1 + True == 2; _ledger.append(1)
assert True + 1 == 2; _ledger.append(1)
assert True + True == 2; _ledger.append(1)
assert True * 3 == 3; _ledger.append(1)

# 3) str same-type
assert "a" + "b" == "ab"; _ledger.append(1)
assert "1" + "2" == "12"; _ledger.append(1)
assert "a" * 3 == "aaa"; _ledger.append(1)
assert 3 * "a" == "aaa"; _ledger.append(1)
assert str(1) + "a" == "1a"; _ledger.append(1)
assert "a" + str(1) == "a1"; _ledger.append(1)

# 4) bytes same-type
assert b"a" + b"b" == b"ab"; _ledger.append(1)
assert b"a" * 3 == b"aaa"; _ledger.append(1)

# 5) list/tuple same-type concatenation
assert [1] + [2] == [1, 2]; _ledger.append(1)
assert (1,) + (2,) == (1, 2); _ledger.append(1)
assert [1] * 3 == [1, 1, 1]; _ledger.append(1)
assert (1,) * 3 == (1, 1, 1); _ledger.append(1)

# 6) set ops
assert {1, 2} | {3} == {1, 2, 3}; _ledger.append(1)
assert {1, 2} & {2, 3} == {2}; _ledger.append(1)
assert {1, 2} - {1} == {2}; _ledger.append(1)
assert {1, 2} ^ {2, 3} == {1, 3}; _ledger.append(1)

# 7) dict | merge
assert {1: "a"} | {2: "b"} == {1: "a", 2: "b"}; _ledger.append(1)

# 8) string methods
assert "abc".upper() == "ABC"; _ledger.append(1)
assert "  x  ".strip() == "x"; _ledger.append(1)
assert "a,b,c".split(",") == ["a", "b", "c"]; _ledger.append(1)
assert " ".join(["a", "b", "c"]) == "a b c"; _ledger.append(1)
assert "abc".replace("a", "x") == "xbc"; _ledger.append(1)
assert "abc".startswith("a") == True; _ledger.append(1)

# 9) float/int coercion
assert int(1.5) == 1; _ledger.append(1)
assert int("5") == 5; _ledger.append(1)
assert float("1.5") == 1.5; _ledger.append(1)
assert float(1) == 1.0; _ledger.append(1)

# 10) bool() truth-value
assert bool(0) == False; _ledger.append(1)
assert bool("") == False; _ledger.append(1)
assert bool([]) == False; _ledger.append(1)
assert bool(1) == True; _ledger.append(1)
assert bool("x") == True; _ledger.append(1)
assert bool([0]) == True; _ledger.append(1)

# 11) unary +/-/~ on numerics
assert -1 == 0 - 1; _ledger.append(1)
assert +1 == 1; _ledger.append(1)
assert ~0 == -1; _ledger.append(1)
assert ~5 == -6; _ledger.append(1)
assert -True == -1; _ledger.append(1)
assert +True == 1; _ledger.append(1)

# 12) eval() with same-type ops still works on both runtimes
assert eval("1 + 1.0") == 2.0; _ledger.append(1)
assert eval("'a' + 'b'") == "ab"; _ledger.append(1)
assert eval("[1] + [2]") == [1, 2]; _ledger.append(1)
assert eval("(1,) + (2,)") == (1, 2); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_lang_arith_string_concat_value_ops {sum(_ledger)} asserts")
