# Atomic 323 pass conformance — built-in collection method depth:
# list (count/index/append/extend/insert/remove/pop/sort/reverse/
# copy), dict (keys/values/items/get/setdefault/update/pop/clear/
# copy/in), set (add/remove/discard/union/intersection/difference/
# symmetric_difference/issubset/issuperset/isdisjoint), str (title/
# lower/upper/swapcase/capitalize/casefold/count/find/rfind/ljust/
# rjust/center/zfill/is*/partition/rpartition/rsplit/splitlines/
# lstrip/rstrip/strip-chars/startswith-tuple/endswith-tuple/remove
# prefix/removesuffix/replace-count/find-with-bounds), bytes
# (split/join/startswith/replace/hex/fromhex/count/find), iter/
# next/StopIteration, divmod and floor-div/modulo on negatives,
# and container-method exception protocols (list.index/remove
# ValueError, set.remove KeyError, dict.pop/dict[k] KeyError,
# next() StopIteration). All asserts match between CPython 3.12
# and mamba.

_ledger: list[int] = []

# 1) list methods
assert [1, 2, 2, 3].count(2) == 2; _ledger.append(1)
assert [1, 2, 3].index(2) == 1; _ledger.append(1)
_lst = [1, 2, 3]
_lst.append(4)
assert _lst == [1, 2, 3, 4]; _ledger.append(1)
_lst2 = [1, 2, 3]
_lst2.extend([4, 5])
assert _lst2 == [1, 2, 3, 4, 5]; _ledger.append(1)
_lst3 = [1, 2, 3]
_lst3.insert(1, 99)
assert _lst3 == [1, 99, 2, 3]; _ledger.append(1)
_lst4 = [1, 2, 3, 2]
_lst4.remove(2)
assert _lst4 == [1, 3, 2]; _ledger.append(1)
_lst5 = [1, 2, 3]
assert _lst5.pop() == 3; _ledger.append(1)
assert _lst5 == [1, 2]; _ledger.append(1)
_lst6 = [1, 2, 3]
assert _lst6.pop(0) == 1; _ledger.append(1)
assert _lst6 == [2, 3]; _ledger.append(1)
_lst7 = [3, 1, 2]
_lst7.sort()
assert _lst7 == [1, 2, 3]; _ledger.append(1)
_lst8 = [3, 1, 2]
_lst8.sort(reverse=True)
assert _lst8 == [3, 2, 1]; _ledger.append(1)
_lst9 = [1, 2, 3]
_lst9.reverse()
assert _lst9 == [3, 2, 1]; _ledger.append(1)
assert [1, 2, 3].copy() == [1, 2, 3]; _ledger.append(1)

# 2) dict methods
assert list({"a": 1, "b": 2}.keys()) == ["a", "b"]; _ledger.append(1)
assert sorted({"a": 1, "b": 2}.values()) == [1, 2]; _ledger.append(1)
assert sorted({"a": 1, "b": 2}.items()) == [("a", 1), ("b", 2)]; _ledger.append(1)
assert {"a": 1}.get("a") == 1; _ledger.append(1)
assert {"a": 1}.get("x", "d") == "d"; _ledger.append(1)
assert {"a": 1}.get("x") is None; _ledger.append(1)
_d = {"a": 1}
_d.setdefault("a", 99)
_d.setdefault("b", 2)
assert _d == {"a": 1, "b": 2}; _ledger.append(1)
_d2 = {"a": 1}
_d2.update({"b": 2, "a": 99})
assert _d2 == {"a": 99, "b": 2}; _ledger.append(1)
_d3 = {"a": 1, "b": 2}
assert _d3.pop("a") == 1; _ledger.append(1)
assert _d3 == {"b": 2}; _ledger.append(1)
_d4 = {"a": 1, "b": 2}
_d4.clear()
assert _d4 == {}; _ledger.append(1)
assert {"a": 1}.copy() == {"a": 1}; _ledger.append(1)
assert ("a" in {"a": 1}) == True; _ledger.append(1)
assert ("x" in {"a": 1}) == False; _ledger.append(1)

# 3) set methods
_s = {1, 2}
_s.add(3)
assert _s == {1, 2, 3}; _ledger.append(1)
_s2 = {1, 2, 3}
_s2.remove(2)
assert _s2 == {1, 3}; _ledger.append(1)
_s3 = {1, 2, 3}
_s3.discard(99)
assert _s3 == {1, 2, 3}; _ledger.append(1)
assert {1, 2}.union({3, 4}) == {1, 2, 3, 4}; _ledger.append(1)
assert {1, 2, 3}.intersection({2, 3, 4}) == {2, 3}; _ledger.append(1)
assert {1, 2, 3}.difference({2}) == {1, 3}; _ledger.append(1)
assert {1, 2}.symmetric_difference({2, 3}) == {1, 3}; _ledger.append(1)
assert {1, 2}.issubset({1, 2, 3}) == True; _ledger.append(1)
assert {1, 2, 5}.issubset({1, 2, 3}) == False; _ledger.append(1)
assert {1, 2, 3}.issuperset({1, 2}) == True; _ledger.append(1)
assert {1, 2}.isdisjoint({3, 4}) == True; _ledger.append(1)
assert {1, 2}.isdisjoint({2, 3}) == False; _ledger.append(1)

# 4) str methods depth
assert "hello world".title() == "Hello World"; _ledger.append(1)
assert "AbC".lower() == "abc"; _ledger.append(1)
assert "AbC".upper() == "ABC"; _ledger.append(1)
assert "AbC".swapcase() == "aBc"; _ledger.append(1)
assert "abc".capitalize() == "Abc"; _ledger.append(1)
assert "ABc".casefold() == "abc"; _ledger.append(1)
assert "abca".count("a") == 2; _ledger.append(1)
assert "abc".find("b") == 1; _ledger.append(1)
assert "abc".find("z") == -1; _ledger.append(1)
assert "abca".rfind("a") == 3; _ledger.append(1)
assert "a".ljust(5) == "a    "; _ledger.append(1)
assert "a".rjust(5) == "    a"; _ledger.append(1)
assert "a".center(5) == "  a  "; _ledger.append(1)
assert "42".zfill(5) == "00042"; _ledger.append(1)
assert "abc".isalpha() == True; _ledger.append(1)
assert "abc123".isalpha() == False; _ledger.append(1)
assert "123".isdigit() == True; _ledger.append(1)
assert "abc123".isalnum() == True; _ledger.append(1)
assert " ".isspace() == True; _ledger.append(1)
assert "abc".isascii() == True; _ledger.append(1)
assert "abc".islower() == True; _ledger.append(1)
assert "ABC".isupper() == True; _ledger.append(1)
assert "a/b/c".partition("/") == ("a", "/", "b/c"); _ledger.append(1)
assert "a/b/c".rpartition("/") == ("a/b", "/", "c"); _ledger.append(1)
assert "a,b,c".rsplit(",", 1) == ["a,b", "c"]; _ledger.append(1)
assert "a\nb\nc".splitlines() == ["a", "b", "c"]; _ledger.append(1)
assert " x ".lstrip() == "x "; _ledger.append(1)
assert " x ".rstrip() == " x"; _ledger.append(1)
assert "__a__".strip("_") == "a"; _ledger.append(1)
assert "abc".startswith(("a", "b")) == True; _ledger.append(1)
assert "abc".endswith(("c", "d")) == True; _ledger.append(1)
assert "abc".removeprefix("ab") == "c"; _ledger.append(1)
assert "abc".removesuffix("bc") == "a"; _ledger.append(1)
assert "aabc".replace("a", "X", 1) == "Xabc"; _ledger.append(1)
assert "abc".find("b", 1, 3) == 1; _ledger.append(1)

# 5) bytes methods (subset that mamba supports)
assert b"a,b".split(b",") == [b"a", b"b"]; _ledger.append(1)
assert b" ".join([b"a", b"b"]) == b"a b"; _ledger.append(1)
assert b"abc".startswith(b"a") == True; _ledger.append(1)
assert b"abc".replace(b"a", b"X") == b"Xbc"; _ledger.append(1)
assert b"\x61\x62".hex() == "6162"; _ledger.append(1)
assert bytes.fromhex("6162") == b"ab"; _ledger.append(1)
assert b"abc".count(b"a") == 1; _ledger.append(1)
assert b"abc".find(b"b") == 1; _ledger.append(1)

# 6) iter/next/StopIteration
_it = iter([1, 2])
assert next(_it) == 1; _ledger.append(1)
assert next(_it) == 2; _ledger.append(1)
_empty_it = iter([])
_raised = False
try:
    next(_empty_it)
except StopIteration:
    _raised = True
assert _raised == True; _ledger.append(1)
assert next(iter([]), "def") == "def"; _ledger.append(1)

# 7) numeric edge cases (excluding 2**-1 / 0**-1 which diverge)
assert divmod(-7, 3) == (-3, 2); _ledger.append(1)
assert (-7) // 3 == -3; _ledger.append(1)
assert (-7) % 3 == 2; _ledger.append(1)
assert (-2) ** 3 == -8; _ledger.append(1)
assert complex(1, 2) == 1 + 2j; _ledger.append(1)
assert (1 + 2j) + (3 + 4j) == 4 + 6j; _ledger.append(1)

# 8) container exception protocols
_index_raised = False
try:
    [1, 2].index(99)
except ValueError:
    _index_raised = True
assert _index_raised == True; _ledger.append(1)

_remove_raised = False
try:
    [1, 2].remove(99)
except ValueError:
    _remove_raised = True
assert _remove_raised == True; _ledger.append(1)

_set_remove_raised = False
try:
    {1}.remove(99)
except KeyError:
    _set_remove_raised = True
assert _set_remove_raised == True; _ledger.append(1)

_dict_pop_raised = False
try:
    {}.pop("k")
except KeyError:
    _dict_pop_raised = True
assert _dict_pop_raised == True; _ledger.append(1)

_dict_get_raised = False
try:
    {}["k"]
except KeyError:
    _dict_get_raised = True
assert _dict_get_raised == True; _ledger.append(1)

_next_empty_raised = False
try:
    next(iter([]))
except StopIteration:
    _next_empty_raised = True
assert _next_empty_raised == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_lang_collection_methods_value_ops {sum(_ledger)} asserts")
