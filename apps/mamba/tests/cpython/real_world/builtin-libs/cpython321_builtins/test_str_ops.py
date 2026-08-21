# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_str_ops"
# subject = "cpython321.test_str_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_str_ops.py"
# status = "filled"
# ///
"""cpython321.test_str_ops: execute CPython 3.12 seed test_str_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for builtin `str`.
# Surface: indexing/slicing, len, concat, repeat, upper/lower/title,
# strip/lstrip/rstrip, split/rsplit/splitlines, join, startswith/
# endswith, find/index/count, replace, format/f-string, in/not in.
# Companion to stub/test_str.py — vendored unittest seed.
_ledger: list[int] = []
s = "hello world"
assert len(s) == 11; _ledger.append(1)
assert s[0] == "h"; _ledger.append(1)
assert s[-1] == "d"; _ledger.append(1)
assert s[0:5] == "hello"; _ledger.append(1)
assert s.upper() == "HELLO WORLD"; _ledger.append(1)
assert "ABC".lower() == "abc"; _ledger.append(1)
assert "hello world".title() == "Hello World"; _ledger.append(1)
assert "  spaced  ".strip() == "spaced"; _ledger.append(1)
assert "xxhelloxx".lstrip("x") == "helloxx"; _ledger.append(1)
assert "xxhelloxx".rstrip("x") == "xxhello"; _ledger.append(1)
assert "a,b,c".split(",") == ["a", "b", "c"]; _ledger.append(1)
assert "a b c".split() == ["a", "b", "c"]; _ledger.append(1)
assert "line1\nline2\n".splitlines() == ["line1", "line2"]; _ledger.append(1)
assert ",".join(["a", "b", "c"]) == "a,b,c"; _ledger.append(1)
assert s.startswith("hello"); _ledger.append(1)
assert s.endswith("world"); _ledger.append(1)
assert s.find("world") == 6; _ledger.append(1)
assert s.find("missing") == -1; _ledger.append(1)
assert "abcabc".count("a") == 2; _ledger.append(1)
assert "abc".replace("b", "X") == "aXc"; _ledger.append(1)
assert "hi" + " " + "there" == "hi there"; _ledger.append(1)
assert "ab" * 3 == "ababab"; _ledger.append(1)
assert "ell" in s; _ledger.append(1)
assert "xyz" not in s; _ledger.append(1)
name = "world"
assert f"hello, {name}" == "hello, world"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_str_ops {sum(_ledger)} asserts")
