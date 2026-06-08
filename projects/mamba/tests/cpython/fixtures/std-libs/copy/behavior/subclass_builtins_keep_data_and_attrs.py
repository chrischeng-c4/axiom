# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "subclass_builtins_keep_data_and_attrs"
# subject = "copy.deepcopy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.deepcopy: list/dict/tuple subclasses keep both their element data and extra instance attributes through shallow (shared inners) and deep (independent) copies"""
import copy


# list subclass with an extra instance attribute.
class MyList(list):
    pass


ml = MyList([[1, 2], 3])
ml.foo = [4, 5]
ml_s = copy.copy(ml)
assert list(ml_s) == list(ml), "list subclass shallow elements equal"
assert ml_s[0] is ml[0] and ml_s.foo is ml.foo, "list subclass shallow shares inners + attr"
ml_d = copy.deepcopy(ml)
assert list(ml_d) == list(ml) and ml_d.foo == ml.foo, "list subclass deep equal"
assert ml_d[0] is not ml[0] and ml_d.foo is not ml.foo, "list subclass deep independent"


# dict subclass with private bookkeeping.
class MyDict(dict):
    def __init__(self, d=None):
        d = d or {}
        self._keys = list(d.keys())
        super().__init__(d)

    def __setitem__(self, key, item):
        super().__setitem__(key, item)
        if key not in self._keys:
            self._keys.append(key)


md = MyDict(d={"foo": 0})
md_d = copy.deepcopy(md)
assert md == md_d and md._keys == md_d._keys and md is not md_d, "dict subclass deep copy"
md["bar"] = 1
assert md != md_d and md._keys != md_d._keys, "deep copy stays independent after mutation"


# tuple subclass keeps its element data through copy and deepcopy.
class MyTuple(tuple):
    pass


mt = MyTuple([[1, 2], 3])
assert tuple(copy.copy(mt)) == ([1, 2], 3), "tuple subclass shallow elements"
mt_d = copy.deepcopy(mt)
assert tuple(mt_d) == ([1, 2], 3) and mt_d is not mt, "tuple subclass deep new"
assert mt_d[0] is not mt[0], "tuple subclass deep copies inner list"

print("subclass_builtins_keep_data_and_attrs OK")
