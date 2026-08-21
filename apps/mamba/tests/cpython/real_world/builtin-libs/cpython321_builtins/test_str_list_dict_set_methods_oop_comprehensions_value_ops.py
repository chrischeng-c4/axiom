# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_str_list_dict_set_methods_oop_comprehensions_value_ops"
# subject = "cpython321.test_str_list_dict_set_methods_oop_comprehensions_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_str_list_dict_set_methods_oop_comprehensions_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_str_list_dict_set_methods_oop_comprehensions_value_ops: execute CPython 3.12 seed test_str_list_dict_set_methods_oop_comprehensions_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 249 pass conformance — str deep methods (split/rsplit/splitlines/
# strip/lstrip/rstrip/join/replace/find/rfind/index/count/upper/lower/
# title/capitalize/swapcase/casefold/isalpha/isdigit/isspace/isalnum/
# isupper/islower/istitle/startswith/endswith/center/ljust/rjust/zfill/
# partition/rpartition/expandtabs/removeprefix/removesuffix/encode/
# maketrans/translate) + str format spec edge cases (:b /:o /:#x /:, /
# :+d /:%) / list methods (append/extend/insert/remove/reverse/sort/
# count/index/copy/clear) / dict methods (keys/values/items/get/
# get-default/update/setdefault/in/not-in) / set methods (add/remove/
# discard/union/intersection/difference/symmetric_difference/issubset/
# issuperset/isdisjoint) / isinstance int/str/list/tuple + multi /
# issubclass bool-int/list-list/dict-object / repr dict/list/tuple
# insertion order / OOP (instance attr lookup, classmethod,
# staticmethod, property, super()) / dict constructor variants
# (dict(), dict(a=1), dict(from pairs), dict(from zip)) /
# comprehensions (list, list-cond, list-nested, dict, set, gen-expr)
# that match between CPython 3.12 and mamba.


class _MyCls:
    cv = 99
    def m(self):
        return 1


class _Base:
    def f(self):
        return "base"


class _Sub(_Base):
    def f(self):
        return "sub-" + super().f()


class _PropCls:
    @property
    def value(self):
        return 42


class _CM:
    @classmethod
    def cm(cls):
        return cls.__name__
    @staticmethod
    def sm():
        return "static"


_ledger: list[int] = []

# 1) str split/strip/join
assert "a b c".split() == ["a", "b", "c"]; _ledger.append(1)
assert "a,b,c".split(",") == ["a", "b", "c"]; _ledger.append(1)
assert "a,b,c".split(",", 1) == ["a", "b,c"]; _ledger.append(1)
assert "a,b,c".rsplit(",", 1) == ["a,b", "c"]; _ledger.append(1)
assert "a\nb\nc".splitlines() == ["a", "b", "c"]; _ledger.append(1)
assert "  hi  ".strip() == "hi"; _ledger.append(1)
assert "xxhixx".strip("x") == "hi"; _ledger.append(1)
assert "  hi  ".lstrip() == "hi  "; _ledger.append(1)
assert "  hi  ".rstrip() == "  hi"; _ledger.append(1)
assert ",".join(["a", "b", "c"]) == "a,b,c"; _ledger.append(1)
assert "".join(["a", "b", "c"]) == "abc"; _ledger.append(1)

# 2) str replace/find/index/count
assert "hello".replace("l", "L") == "heLLo"; _ledger.append(1)
assert "hello".replace("l", "L", 1) == "heLlo"; _ledger.append(1)
assert "hello".find("l") == 2; _ledger.append(1)
assert "hello".find("z") == -1; _ledger.append(1)
assert "hello".rfind("l") == 3; _ledger.append(1)
assert "hello".index("l") == 2; _ledger.append(1)
assert "hello".count("l") == 2; _ledger.append(1)

# 3) str case
assert "hi".upper() == "HI"; _ledger.append(1)
assert "HI".lower() == "hi"; _ledger.append(1)
assert "hello world".title() == "Hello World"; _ledger.append(1)
assert "hello".capitalize() == "Hello"; _ledger.append(1)
assert "Hi There".swapcase() == "hI tHERE"; _ledger.append(1)
assert "HELLO".casefold() == "hello"; _ledger.append(1)

# 4) str predicates
assert "abc".isalpha() == True; _ledger.append(1)
assert "ab1".isalpha() == False; _ledger.append(1)
assert "123".isdigit() == True; _ledger.append(1)
assert "12a".isdigit() == False; _ledger.append(1)
assert "   ".isspace() == True; _ledger.append(1)
assert "abc123".isalnum() == True; _ledger.append(1)
assert "ABC".isupper() == True; _ledger.append(1)
assert "abc".islower() == True; _ledger.append(1)
assert "Hello World".istitle() == True; _ledger.append(1)

# 5) str startswith/endswith
assert "hello".startswith("he") == True; _ledger.append(1)
assert "hello".startswith("lo") == False; _ledger.append(1)
assert "hello".startswith(("xx", "he")) == True; _ledger.append(1)
assert "hello".endswith("lo") == True; _ledger.append(1)
assert "hello".endswith("he") == False; _ledger.append(1)
assert "hello".endswith(("xx", "lo")) == True; _ledger.append(1)

# 6) str pad/partition/expand/encode
assert "x".center(5) == "  x  "; _ledger.append(1)
assert "x".center(5, "*") == "**x**"; _ledger.append(1)
assert "x".ljust(5) == "x    "; _ledger.append(1)
assert "x".rjust(5) == "    x"; _ledger.append(1)
assert "42".zfill(5) == "00042"; _ledger.append(1)
assert "a=b=c".partition("=") == ("a", "=", "b=c"); _ledger.append(1)
assert "a=b=c".rpartition("=") == ("a=b", "=", "c"); _ledger.append(1)
assert "a\tb".expandtabs(4) == "a   b"; _ledger.append(1)
assert "TestHello".removeprefix("Test") == "Hello"; _ledger.append(1)
assert "HelloTest".removesuffix("Test") == "Hello"; _ledger.append(1)
assert "café".encode("utf-8") == b"caf\xc3\xa9"; _ledger.append(1)
assert "hi".encode("ascii") == b"hi"; _ledger.append(1)

# 7) str.maketrans / translate
assert type(str.maketrans("abc", "xyz")).__name__ == "dict"; _ledger.append(1)
assert "abc".translate(str.maketrans("abc", "xyz")) == "xyz"; _ledger.append(1)

# 8) str format spec edge cases
assert "{:b}".format(10) == "1010"; _ledger.append(1)
assert "{:o}".format(8) == "10"; _ledger.append(1)
assert "{:#x}".format(255) == "0xff"; _ledger.append(1)
assert "{:,}".format(1234567) == "1,234,567"; _ledger.append(1)
assert "{:+d}".format(42) == "+42"; _ledger.append(1)
assert "{:%}".format(0.25) == "25.000000%"; _ledger.append(1)

# 9) list methods — mutation
_L: list = [1, 2, 3]
_L.append(4); assert _L == [1, 2, 3, 4]; _ledger.append(1)
_L2: list = [1, 2]
_L2.extend([4, 5]); assert _L2 == [1, 2, 4, 5]; _ledger.append(1)
_L3: list = [1, 2, 3]
_L3.insert(1, 99); assert _L3 == [1, 99, 2, 3]; _ledger.append(1)
_L4: list = [1, 2, 3]
_L4.remove(2); assert _L4 == [1, 3]; _ledger.append(1)
_L5: list = [1, 2, 3]
_L5.reverse(); assert _L5 == [3, 2, 1]; _ledger.append(1)
_L6: list = [3, 1, 2]
_L6.sort(); assert _L6 == [1, 2, 3]; _ledger.append(1)
assert [1, 2, 1, 3, 1].count(1) == 3; _ledger.append(1)
assert [10, 20, 30].index(20) == 1; _ledger.append(1)
assert [1, 2, 3].copy() == [1, 2, 3]; _ledger.append(1)
_L7: list = [1, 2, 3]
_L7.clear(); assert _L7 == []; _ledger.append(1)

# 10) dict methods
assert list({"a": 1, "b": 2}.keys()) == ["a", "b"]; _ledger.append(1)
assert list({"a": 1, "b": 2}.values()) == [1, 2]; _ledger.append(1)
assert list({"a": 1, "b": 2}.items()) == [("a", 1), ("b", 2)]; _ledger.append(1)
assert {"a": 1}.get("a") == 1; _ledger.append(1)
assert {"a": 1}.get("z", -1) == -1; _ledger.append(1)
_D: dict = {"a": 1}
_D.update({"c": 3}); assert _D == {"a": 1, "c": 3}; _ledger.append(1)
assert {"a": 1}.setdefault("b", 99) == 99; _ledger.append(1)
assert ("a" in {"a": 1}) == True; _ledger.append(1)
assert ("z" in {"a": 1}) == False; _ledger.append(1)

# 11) set methods
_S: set = {1, 2, 3}
_S.add(4); assert sorted(_S) == [1, 2, 3, 4]; _ledger.append(1)
_S2: set = {1, 2, 3}
_S2.remove(2); assert sorted(_S2) == [1, 3]; _ledger.append(1)
_S3: set = {1, 2, 3}
_S3.discard(99); assert sorted(_S3) == [1, 2, 3]; _ledger.append(1)
assert sorted({1, 2}.union({3, 4})) == [1, 2, 3, 4]; _ledger.append(1)
assert sorted({1, 2, 3}.intersection({2, 3, 4})) == [2, 3]; _ledger.append(1)
assert sorted({1, 2, 3}.difference({2, 3})) == [1]; _ledger.append(1)
assert sorted({1, 2, 3}.symmetric_difference({2, 3, 4})) == [1, 4]; _ledger.append(1)
assert {1, 2}.issubset({1, 2, 3}) == True; _ledger.append(1)
assert {1, 2, 3}.issuperset({1, 2}) == True; _ledger.append(1)
assert {1, 2}.isdisjoint({3, 4}) == True; _ledger.append(1)

# 12) isinstance / issubclass — type-name accepting
assert isinstance(42, int) == True; _ledger.append(1)
assert isinstance("hi", str) == True; _ledger.append(1)
assert isinstance([1, 2], list) == True; _ledger.append(1)
assert isinstance((), tuple) == True; _ledger.append(1)
assert isinstance(42, (int, str)) == True; _ledger.append(1)
assert issubclass(bool, int) == True; _ledger.append(1)
assert issubclass(list, list) == True; _ledger.append(1)
assert issubclass(dict, object) == True; _ledger.append(1)

# 13) repr insertion order
assert repr({"b": 1, "a": 2, "c": 3}) == "{'b': 1, 'a': 2, 'c': 3}"; _ledger.append(1)
assert repr([1, [2, 3], "x"]) == "[1, [2, 3], 'x']"; _ledger.append(1)
assert repr((1, "x", [2])) == "(1, 'x', [2])"; _ledger.append(1)

# 14) OOP — instance attr / classmethod / staticmethod / property / super
assert _MyCls().m() == 1; _ledger.append(1)
assert _MyCls.cv == 99; _ledger.append(1)
assert _MyCls().cv == 99; _ledger.append(1)
assert _Sub().f() == "sub-base"; _ledger.append(1)
assert _PropCls().value == 42; _ledger.append(1)
assert _CM.cm() == "_CM"; _ledger.append(1)
assert _CM.sm() == "static"; _ledger.append(1)

# 15) dict constructor variants
assert dict() == {}; _ledger.append(1)
assert dict(a=1) == {"a": 1}; _ledger.append(1)
assert dict([("a", 1)]) == {"a": 1}; _ledger.append(1)
assert dict(zip("ab", [1, 2])) == {"a": 1, "b": 2}; _ledger.append(1)

# 16) comprehensions
assert [x * 2 for x in range(4)] == [0, 2, 4, 6]; _ledger.append(1)
assert [x for x in range(10) if x % 2 == 0] == [0, 2, 4, 6, 8]; _ledger.append(1)
assert [(a, b) for a in [1, 2] for b in [10, 20]] == [(1, 10), (1, 20), (2, 10), (2, 20)]; _ledger.append(1)
assert {k: k * 2 for k in range(3)} == {0: 0, 1: 2, 2: 4}; _ledger.append(1)
assert sorted({x % 3 for x in range(10)}) == [0, 1, 2]; _ledger.append(1)
assert sum(x * 2 for x in range(4)) == 12; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_str_list_dict_set_methods_oop_comprehensions_value_ops {sum(_ledger)} asserts")
