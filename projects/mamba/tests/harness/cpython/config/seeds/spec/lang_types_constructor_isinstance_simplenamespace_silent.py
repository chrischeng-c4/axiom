# Operational AssertionPass seed for SILENT divergences in the `types`
# module: every value-producing constructor (SimpleNamespace, ModuleType,
# MappingProxyType, NoneType), the runtime identity of generator
# expressions, the documented type-alias identity of LambdaType and
# NoneType, the isinstance hooks against types.GeneratorType /
# FunctionType / LambdaType, and the `types.resolve_bases` helper.
#
# The matching subset (every `types.X.__name__` returning the documented
# type-name string, plus `hasattr(types, X)` being True) is covered by
# `test_types_class_descriptor_name_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • types.SimpleNamespace(a=1).a — instance attribute access
#     (mamba: AttributeError because `SimpleNamespace` is unbound on
#     the `types` shim dict);
#   • types.ModuleType("x").__name__ == "x" — runtime module construction
#     (mamba: AttributeError on `types.ModuleType` call);
#   • types.MappingProxyType({1: 2})[1] == 2 — read-only mapping view
#     (mamba: AttributeError on call);
#   • types.NoneType() is None — None singleton constructor
#     (mamba: AttributeError on call);
#   • types.NoneType is type(None) — alias identity
#     (mamba: returns False, separate stub instance);
#   • types.LambdaType is types.FunctionType — documented alias identity
#     (mamba: returns False, separate stub instance);
#   • type((x for x in [1])).__name__ == "generator" — runtime type of
#     a generator expression (mamba: returns "list" — generator
#     expressions are materialised eagerly);
#   • isinstance(<gen>, types.GeneratorType) — generator instance check
#     (mamba: False, isinstance hook not wired through the stub);
#   • isinstance(<fn>, types.FunctionType) — function instance check
#     (mamba: False);
#   • isinstance(<lambda>, types.LambdaType) — lambda instance check
#     (mamba: False);
#   • types.resolve_bases — module-level helper
#     (mamba: attribute resolves to None, not a callable).
import types
from typing import Any

_ledger: list[int] = []

# 1) types.SimpleNamespace — keyword-argument attribute construction
_ns: Any = types.SimpleNamespace(a=1, b="two")
assert _ns.a == 1; _ledger.append(1)
assert _ns.b == "two"; _ledger.append(1)
assert type(_ns).__name__ == "SimpleNamespace"; _ledger.append(1)

# 2) types.ModuleType("x") — runtime module construction
_m: Any = types.ModuleType("custom_mod")
assert _m.__name__ == "custom_mod"; _ledger.append(1)
assert type(_m).__name__ == "module"; _ledger.append(1)
assert isinstance(_m, types.ModuleType); _ledger.append(1)

# 3) types.MappingProxyType — read-only mapping view
_mp: Any = types.MappingProxyType({1: 2, 3: 4})
assert _mp[1] == 2; _ledger.append(1)
assert _mp[3] == 4; _ledger.append(1)
assert len(_mp) == 2; _ledger.append(1)
assert type(_mp).__name__ == "mappingproxy"; _ledger.append(1)

# 4) types.NoneType — None singleton constructor
_n: Any = types.NoneType()
assert _n is None; _ledger.append(1)

# 5) Documented alias identities
assert types.NoneType is type(None); _ledger.append(1)
assert types.LambdaType is types.FunctionType; _ledger.append(1)

# 6) Generator-expression runtime type
def _g():
    yield 1
    yield 2

_gen: Any = _g()
assert type(_gen).__name__ == "generator"; _ledger.append(1)
_genexpr: Any = (x for x in [1, 2, 3])
assert type(_genexpr).__name__ == "generator"; _ledger.append(1)

# 7) isinstance hooks against the `types` tokens
assert isinstance(_g(), types.GeneratorType); _ledger.append(1)

def _myfn(a: int, b: int) -> int:
    return a + b

assert isinstance(_myfn, types.FunctionType); _ledger.append(1)

_lam: Any = lambda x: x
assert isinstance(_lam, types.LambdaType); _ledger.append(1)
assert isinstance(_lam, types.FunctionType); _ledger.append(1)

# 8) types.resolve_bases — module-level helper for PEP-560 class bases
assert callable(types.resolve_bases); _ledger.append(1)
# Identity transform: passing a tuple of plain types yields the same
# tuple back when there are no PEP-560 __mro_entries__.
_bases: Any = types.resolve_bases((int, str))
assert _bases == (int, str); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_types_constructor_isinstance_simplenamespace_silent {sum(_ledger)} asserts")
