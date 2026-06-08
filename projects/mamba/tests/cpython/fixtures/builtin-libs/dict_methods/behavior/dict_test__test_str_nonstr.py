# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "dict_methods"
# dimension = "behavior"
# case = "dict_test__test_str_nonstr"
# subject = "cpython.test_dict.DictTest.test_str_nonstr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: DictTest::test_str_nonstr (CPython 3.12 oracle)."""

import sys


def run():
    class StrSub(str):
        pass

    eq_count = 0

    class Key3:
        def __hash__(self):
            return hash("key3")

        def __eq__(self, other):
            nonlocal eq_count
            if isinstance(other, Key3) or isinstance(other, str) and other == "key3":
                eq_count += 1
                return True
            return False

    key3_1 = StrSub("key3")
    key3_2 = Key3()
    key3_3 = Key3()

    dicts = []

    for key3 in (key3_1, key3_2):
        dicts.append({"key1": 42, "key2": 43, key3: 44})

        d = {"key1": 42, "key2": 43}
        d[key3] = 44
        dicts.append(d)

        d = {"key1": 42, "key2": 43}
        assert d.setdefault(key3, 44) == 44
        dicts.append(d)

        d = {"key1": 42, "key2": 43}
        d.update({key3: 44})
        dicts.append(d)

        d = {"key1": 42, "key2": 43}
        d |= {key3: 44}
        dicts.append(d)

        def make_pairs():
            yield ("key1", 42)
            yield ("key2", 43)
            yield (key3, 44)

        d = dict(make_pairs())
        dicts.append(d)

        d = d.copy()
        dicts.append(d)

        d = {key: 42 + i for i, key in enumerate(["key1", "key2", key3])}
        dicts.append(d)

    for d in dicts:
        assert d.get("key1") == 42

        noninterned_key1 = "ke"
        noninterned_key1 += "y1"
        if sys.implementation.name == "cpython":
            interned_key1 = "key1"
            assert noninterned_key1 is not interned_key1
        assert d.get(noninterned_key1) == 42

        assert d.get("key3") == 44
        assert d.get(key3_1) == 44
        assert d.get(key3_2) == 44

        eq_count = 0
        assert d.get(key3_3) == 44
        assert eq_count >= 1


run()
print("DictTest::test_str_nonstr: ok")
