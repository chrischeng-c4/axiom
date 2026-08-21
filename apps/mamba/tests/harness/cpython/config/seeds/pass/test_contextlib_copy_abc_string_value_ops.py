# Operational AssertionPass seed for the value contract of four
# bootstrap stdlib / language surfaces used by every resource-
# management / cloning / abstract-base-class / string-constant
# path: `contextlib` (the documented top-level `suppress` /
# `contextmanager` / `nullcontext` attribute surface +
# `nullcontext` value passthrough), `copy` (the documented
# `copy` / `deepcopy` round-trip semantics on nested lists +
# dict-of-list deep clone independence), `abc` (the documented
# `ABC` / `abstractmethod` / `ABCMeta` attribute surface +
# concrete subclass instantiation contract), and `string`
# (the documented `ascii_letters` / `ascii_lowercase` /
# `ascii_uppercase` / `digits` / `punctuation` / `hexdigits` /
# `octdigits` constant surface + `capwords` title-casing
# helper).
#
# The matching subset between mamba and CPython is the
# contextlib hasattr-surface layer + nullcontext passthrough +
# copy shallow / deep round-trip layer + abc hasattr-surface +
# concrete subclass instantiation layer + string ascii /
# digit / punctuation constant layer + capwords title-casing
# layer.
#
# Surface in this fixture:
#   • contextlib — `suppress`, `contextmanager`, `nullcontext`
#     attribute surface + `nullcontext("x")` value passthrough;
#   • copy — `copy(list)` shallow clone (outer is not, inner
#     is) + `deepcopy(list)` deep clone (outer is not, inner
#     is not) + mutation independence on dict-of-list;
#   • abc — `ABC`, `abstractmethod`, `ABCMeta` attribute
#     surface + concrete subclass overriding `@abstractmethod`
#     is instantiable;
#   • string — `ascii_letters` / `ascii_lowercase` /
#     `ascii_uppercase` / `digits` / `punctuation` /
#     `hexdigits` / `octdigits` constants + `capwords` helper.
#
# Behavioral edges that DIVERGE on mamba (contextlib.ExitStack /
# redirect_stdout / closing absent from documented attribute
# surface, contextlib.suppress does not actually suppress the
# exception, @contextlib.contextmanager yields wrong value,
# abc.ABC abstract class is instantiable in violation of the
# @abstractmethod contract, string.printable len == 0,
# string.Template.substitute / safe_substitute AttributeError
# on 'dict' object) are covered in the matching spec fixture
# `lang_contextlib_abc_string_silent`.
import contextlib
import copy
import abc
import string


_ledger: list[int] = []


class _ConcreteBase(abc.ABC):
    @abc.abstractmethod
    def foo(self) -> str: ...


class _Concrete(_ConcreteBase):
    def foo(self) -> str:
        return "concrete-foo"


# 1) contextlib — module attribute surface
assert hasattr(contextlib, "suppress") == True; _ledger.append(1)
assert hasattr(contextlib, "contextmanager") == True; _ledger.append(1)
assert hasattr(contextlib, "nullcontext") == True; _ledger.append(1)

# 2) contextlib.nullcontext — value passthrough
with contextlib.nullcontext("x") as _n:
    assert _n == "x"; _ledger.append(1)

with contextlib.nullcontext(42) as _n2:
    assert _n2 == 42; _ledger.append(1)

# 3) copy.copy — shallow clone identity contract
_outer = [1, 2, [3, 4]]
_shallow = copy.copy(_outer)
assert _outer == _shallow; _ledger.append(1)
assert (_outer is _shallow) == False; _ledger.append(1)
assert (_outer[2] is _shallow[2]) == True; _ledger.append(1)

# 4) copy.deepcopy — deep clone identity contract
_deep = copy.deepcopy(_outer)
assert _outer == _deep; _ledger.append(1)
assert (_outer is _deep) == False; _ledger.append(1)
assert (_outer[2] is _deep[2]) == False; _ledger.append(1)

# 5) copy.deepcopy — mutation independence
_outer[2].append(99)
assert _shallow[2] == [3, 4, 99]; _ledger.append(1)
assert _deep[2] == [3, 4]; _ledger.append(1)

# 6) copy.deepcopy — dict-of-list independence
_dol = {"a": [1, 2]}
_dol_deep = copy.deepcopy(_dol)
_dol["a"].append(3)
assert _dol_deep["a"] == [1, 2]; _ledger.append(1)

# 7) abc — module attribute surface
assert hasattr(abc, "ABC") == True; _ledger.append(1)
assert hasattr(abc, "abstractmethod") == True; _ledger.append(1)
assert hasattr(abc, "ABCMeta") == True; _ledger.append(1)

# 8) abc — concrete subclass overriding @abstractmethod
_c = _Concrete()
assert _c.foo() == "concrete-foo"; _ledger.append(1)

# 9) string — ascii constant surface
assert string.ascii_letters[:10] == "abcdefghij"; _ledger.append(1)
assert string.ascii_lowercase[:5] == "abcde"; _ledger.append(1)
assert string.ascii_uppercase[:5] == "ABCDE"; _ledger.append(1)
assert len(string.ascii_letters) == 52; _ledger.append(1)
assert len(string.ascii_lowercase) == 26; _ledger.append(1)
assert len(string.ascii_uppercase) == 26; _ledger.append(1)

# 10) string — digit / punctuation / hex / oct constants
assert string.digits == "0123456789"; _ledger.append(1)
assert len(string.digits) == 10; _ledger.append(1)
assert len(string.punctuation) == 32; _ledger.append(1)
assert string.hexdigits == "0123456789abcdefABCDEF"; _ledger.append(1)
assert string.octdigits == "01234567"; _ledger.append(1)

# 11) string.capwords — title-casing helper
assert string.capwords("hello world from mamba") == "Hello World From Mamba"; _ledger.append(1)
assert string.capwords("a b c") == "A B C"; _ledger.append(1)

# NB: contextlib.ExitStack / redirect_stdout / closing absent,
# contextlib.suppress does not suppress the exception,
# @contextlib.contextmanager decorator yields wrong value,
# abc.ABC instantiable in violation of @abstractmethod,
# string.printable len == 0, string.Template.substitute /
# safe_substitute AttributeError on 'dict' object all DIVERGE
# on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_contextlib_copy_abc_string_value_ops {sum(_ledger)} asserts")
