# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "operator_dispatch"
# dimension = "behavior"
# case = "add_module_attr_none_operand_raises"
# subject = "binary + operand type dispatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""Binary `+` type-mismatch check must hard-fault identically whether the
None operand is a local variable or reached via module-attribute access.

A local `x = None; x + 'i'` is caught by mamba's static type checker at
compile time. Reading the same None value through a module attribute
(`mod.attr`) defeats that static narrowing, so the mismatch must instead
be caught at runtime by the `+` dispatcher — this regresses a bug (#1938)
where a None operand reached via attribute access silently returned None
instead of raising, unlike the local-variable case.
"""
import types

mod = types.ModuleType("mamba_regression_1938_mod")
mod.attr = None

try:
    result = mod.attr + "i"
    raise AssertionError(f"expected TypeError, got {result!r}")
except TypeError as e:
    assert str(e) == "unsupported operand type(s) for +: 'NoneType' and 'str'", str(e)

print("add_module_attr_none_operand_raises OK")
