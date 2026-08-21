# Operational AssertionPass seed for the value contract of three
# bootstrap stdlib modules used by every clone / inheritance /
# dynamic-import path: `copy` (the documented shallow vs deep
# clone semantics across `list` / nested-`list` / `dict`), `abc`
# (the abstractmethod decorator + concrete-subclass instantiation
# / abstract-base isinstance contract), and `importlib` (the
# import_module / reload / invalidate_caches / abc / machinery
# helper surface).
#
# The matching subset between mamba and CPython is the clone-
# identity layer + abstract-inheritance layer + import-machinery
# layer: copy.copy(list) returns an equal-but-distinct list;
# copy.copy(nest) keeps inner references (shallow); copy.deepcopy
# (nest) produces independent inner references and remains
# equal; copy.deepcopy(dict) likewise round-trips equal with
# distinct inner refs; abc.abstractmethod is a function;
# hasattr(abc, "abstractproperty" / "abstractclassmethod" /
# "abstractstaticmethod"); a concrete subclass of an
# abstractmethod-decorated abc.ABC can be instantiated and
# isinstance against the abstract base; importlib.import_module
# returns a usable module-like object exposing the expected
# attributes; importlib has the documented reload /
# invalidate_caches / abc / machinery helpers.
#
# Surface in this fixture:
#   • copy.copy([1, 2, 3]) == [1, 2, 3] and is not the source;
#   • shallow copy of nested list shares inner references;
#   • copy.deepcopy of nested list has distinct inner references
#     and equal value;
#   • copy.deepcopy of nested dict has distinct inner refs and
#     equal value;
#   • type(abc.abstractmethod).__name__ == "function";
#   • hasattr(abc, "abstractproperty");
#   • hasattr(abc, "abstractclassmethod");
#   • hasattr(abc, "abstractstaticmethod");
#   • a concrete subclass of an abstractmethod-decorated abc.ABC
#     class can be instantiated and the override is called;
#   • isinstance(_concrete, AbstractBase) is True;
#   • inspect — documented helper surface (isfunction / isclass);
#   • importlib.import_module("json") exposes `dumps`;
#   • hasattr(importlib, "reload");
#   • hasattr(importlib, "invalidate_caches");
#   • hasattr(importlib, "import_module").
#
# Behavioral edges that DIVERGE on mamba (copy.Error class
# identity, abc.ABC / ABCMeta class identity, isinstance(<class>,
# ABCMeta) metaclass predicate, inspect.isfunction INVERTED on
# user-functions vs ints, inspect.signature / Signature /
# Parameter class identity + parameter list, inspect.isbuiltin,
# unicodedata.name English-name format, unicodedata.lookup /
# category / numeric / decimal / digit / normalize /
# bidirectional / east_asian_width helpers,
# type(importlib.import_module(...)) == "module" — mamba returns
# `dict`) are covered in
# `lang_abc_inspect_unicodedata_silent`.
import copy
import abc
import inspect
import importlib


# Abstract-base hierarchy declared at module scope to dodge the
# documented mamba quirk where classes defined inside a `try:`
# block aren't visible to the next statement.
class _Shape(abc.ABC):
    @abc.abstractmethod
    def area(self) -> int:
        ...


class _Square(_Shape):
    def area(self) -> int:
        return 4


_ledger: list[int] = []

# 1) copy.copy — flat list shallow round-trip
_a = [1, 2, 3]
_b = copy.copy(_a)
assert _b == _a; _ledger.append(1)
assert _b is not _a; _ledger.append(1)

# 2) copy.copy — shallow keeps inner references
_nest = [[1, 2], [3, 4]]
_sh = copy.copy(_nest)
assert _sh[0] is _nest[0]; _ledger.append(1)

# 3) copy.deepcopy — distinct inner references, equal value
_dp = copy.deepcopy(_nest)
assert _dp[0] is not _nest[0]; _ledger.append(1)
assert _dp == _nest; _ledger.append(1)

# 4) copy.deepcopy on dict — distinct inner refs, equal value
_d = {"a": [1, 2], "b": [3, 4]}
_d2 = copy.deepcopy(_d)
assert _d2 == _d; _ledger.append(1)
assert _d2["a"] is not _d["a"]; _ledger.append(1)

# 5) abc.abstractmethod — documented function decorator
assert type(abc.abstractmethod).__name__ == "function"; _ledger.append(1)

# 6) abc — documented abstract-*method-shape helper bindings
assert hasattr(abc, "abstractproperty"); _ledger.append(1)
assert hasattr(abc, "abstractclassmethod"); _ledger.append(1)
assert hasattr(abc, "abstractstaticmethod"); _ledger.append(1)

# 7) abc — concrete subclass of abstractmethod-decorated ABC
_sq = _Square()
assert _sq.area() == 4; _ledger.append(1)
assert isinstance(_sq, _Shape) == True; _ledger.append(1)

# 8) inspect — documented helper surface
assert hasattr(inspect, "isfunction"); _ledger.append(1)
assert hasattr(inspect, "isclass"); _ledger.append(1)

# 10) importlib.import_module — usable module-like object
_m = importlib.import_module("json")
assert hasattr(_m, "dumps"); _ledger.append(1)
assert hasattr(_m, "loads"); _ledger.append(1)

# 11) importlib — documented helper surface
assert hasattr(importlib, "reload"); _ledger.append(1)
assert hasattr(importlib, "invalidate_caches"); _ledger.append(1)
assert hasattr(importlib, "import_module"); _ledger.append(1)

# 12) hasattr surface — module-level helpers
assert hasattr(copy, "copy"); _ledger.append(1)
assert hasattr(copy, "deepcopy"); _ledger.append(1)
assert hasattr(abc, "ABC"); _ledger.append(1)
assert hasattr(abc, "ABCMeta"); _ledger.append(1)
assert hasattr(abc, "abstractmethod"); _ledger.append(1)

# NB: copy.Error class identity, abc.ABC / ABCMeta class identity,
# isinstance(<class>, ABCMeta) metaclass predicate,
# inspect.isfunction INVERTED behavior, inspect.signature /
# Signature / Parameter class identity + parameter list,
# inspect.isbuiltin, unicodedata.name English-name format,
# unicodedata.lookup / category / numeric / decimal / digit /
# normalize / bidirectional / east_asian_width helpers,
# type(importlib.import_module(...)) == "module" all DIVERGE on
# mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_copy_abc_importlib_value_ops {sum(_ledger)} asserts")
