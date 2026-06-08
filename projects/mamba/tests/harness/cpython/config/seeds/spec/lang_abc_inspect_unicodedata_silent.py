# Operational AssertionPass seed for SILENT divergences across the
# clone / inheritance / reflection / Unicode-data quintet pinned
# by atomic 148: `copy` (the Error bare class identity), `abc`
# (the ABC / ABCMeta bare class identity + the isinstance metaclass
# predicate `isinstance(<class>, ABCMeta) is True`), `inspect`
# (the documented isfunction / ismodule / isbuiltin predicates,
# the signature / Signature / Parameter introspection surface),
# `unicodedata` (the documented Unicode-name format
# "LATIN CAPITAL LETTER A" + lookup / category / numeric /
# decimal / digit / normalize / bidirectional / east_asian_width
# helpers), and `importlib` (the documented
# `type(import_module(...)) == "module"` instance-class contract).
#
# The matching subset (copy.copy / deepcopy on list / nested-list
# / dict, abc.abstractmethod / abstract*method-shape helpers,
# concrete subclass of abc.ABC instantiation + isinstance on
# abstract base, importlib.import_module surface + reload /
# invalidate_caches / import_module helpers) is covered by
# `test_copy_abc_importlib_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • copy.Error.__name__ == "Error" — clone-failure class
#     identity (mamba: returns None);
#   • abc.ABC.__name__ == "ABC" — bare class identity (mamba:
#     None);
#   • abc.ABCMeta.__name__ == "ABCMeta" — abstract-class
#     metaclass identity (mamba: None);
#   • isinstance(<abc.ABC subclass>, abc.ABCMeta) is True —
#     metaclass predicate (mamba: returns False, mamba doesn't
#     recognize ABCMeta as the metaclass);
#   • inspect.isfunction(_user_fn) is True — Python-function
#     predicate (mamba: returns False — INVERTED, treats user
#     functions as non-functions);
#   • inspect.isfunction(42) is False (mamba: returns True —
#     INVERTED, treats ints as functions);
#   • inspect.isclass(<class>) is True — class predicate
#     (mamba: returns False for user classes);
#   • inspect.ismodule(<module>) is True — module predicate
#     (mamba: AttributeError, inspect is a `dict` with no
#     `ismodule`);
#   • inspect.isbuiltin(len) is True — C-builtin predicate
#     (mamba: AttributeError);
#   • str(inspect.signature(<fn>)) ==
#     "(a, b=2, *args, **kwargs)" — signature serialization
#     (mamba: AttributeError);
#   • list(inspect.signature(<fn>).parameters.keys()) ==
#     ["a", "b", "args", "kwargs"] (mamba: AttributeError);
#   • inspect.Signature.__name__ == "Signature" (mamba:
#     AttributeError);
#   • inspect.Parameter.__name__ == "Parameter" (mamba:
#     AttributeError);
#   • unicodedata.name("A") == "LATIN CAPITAL LETTER A" —
#     Unicode-database character name (mamba: returns
#     "UNICODE CHAR 0041", a fallback hex format);
#   • unicodedata.name("a") == "LATIN SMALL LETTER A" (mamba:
#     "UNICODE CHAR 0061");
#   • unicodedata.lookup("LATIN SMALL LETTER A") == "a" —
#     name → char lookup (mamba: AttributeError);
#   • unicodedata.category("A") == "Lu" — Unicode general
#     category (mamba: AttributeError);
#   • unicodedata.numeric("1") == 1.0 (mamba: AttributeError);
#   • unicodedata.normalize("NFC", "café") == "café" — Unicode
#     normalization form (mamba: AttributeError);
#   • type(importlib.import_module("json")).__name__ == "module"
#     — instance-class contract (mamba: returns "dict",
#     import_module returns a plain dict).
import copy as _copy_mod
import abc as _abc_mod
import inspect as _inspect_mod
import unicodedata as _unicodedata_mod
import importlib as _importlib_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
copy: Any = _copy_mod
abc: Any = _abc_mod
inspect: Any = _inspect_mod
unicodedata: Any = _unicodedata_mod
importlib: Any = _importlib_mod


# Class declared at module scope to dodge the documented mamba
# quirk where classes defined inside a `try:` block aren't visible
# to the next statement.
class _Box(abc.ABC):
    @abc.abstractmethod
    def size(self):
        ...


class _RealBox(_Box):
    def size(self):
        return 8


def _user_fn(a, b=2, *args, **kwargs):
    return a + b


_ledger: list[int] = []

# 1) copy.Error — clone-failure class identity
assert copy.Error.__name__ == "Error"; _ledger.append(1)

# 2) abc.ABC / ABCMeta — class identity
assert abc.ABC.__name__ == "ABC"; _ledger.append(1)
assert abc.ABCMeta.__name__ == "ABCMeta"; _ledger.append(1)

# 3) abc.ABCMeta — metaclass predicate
assert isinstance(_Box, abc.ABCMeta) == True; _ledger.append(1)

# 4) inspect.isfunction — INVERTED on mamba
assert inspect.isfunction(_user_fn) == True; _ledger.append(1)
assert inspect.isfunction(42) == False; _ledger.append(1)

# 5) inspect.isclass — class predicate
assert inspect.isclass(_Box) == True; _ledger.append(1)

# 6) inspect.ismodule — module predicate
assert inspect.ismodule(_inspect_mod) == True; _ledger.append(1)

# 7) inspect.isbuiltin — C-builtin predicate
assert inspect.isbuiltin(len) == True; _ledger.append(1)

# 8) inspect.signature — signature serialization
_sig = inspect.signature(_user_fn)
assert str(_sig) == "(a, b=2, *args, **kwargs)"; _ledger.append(1)
assert list(_sig.parameters.keys()) == ["a", "b", "args", "kwargs"]; _ledger.append(1)

# 9) inspect.Signature / Parameter — class identity
assert inspect.Signature.__name__ == "Signature"; _ledger.append(1)
assert inspect.Parameter.__name__ == "Parameter"; _ledger.append(1)

# 10) unicodedata.name — Unicode-database English names
assert unicodedata.name("A") == "LATIN CAPITAL LETTER A"; _ledger.append(1)
assert unicodedata.name("a") == "LATIN SMALL LETTER A"; _ledger.append(1)

# 11) unicodedata.lookup — name → char
assert unicodedata.lookup("LATIN SMALL LETTER A") == "a"; _ledger.append(1)

# 12) unicodedata.category — Unicode general categories
assert unicodedata.category("A") == "Lu"; _ledger.append(1)
assert unicodedata.category("1") == "Nd"; _ledger.append(1)
assert unicodedata.category(" ") == "Zs"; _ledger.append(1)

# 13) unicodedata.numeric — Unicode numeric value
assert unicodedata.numeric("1") == 1.0; _ledger.append(1)

# 14) unicodedata.normalize — Unicode normalization form
assert unicodedata.normalize("NFC", "café") == "café"; _ledger.append(1)

# 15) importlib.import_module — instance-class contract
assert type(importlib.import_module("json")).__name__ == "module"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_abc_inspect_unicodedata_silent {sum(_ledger)} asserts")
