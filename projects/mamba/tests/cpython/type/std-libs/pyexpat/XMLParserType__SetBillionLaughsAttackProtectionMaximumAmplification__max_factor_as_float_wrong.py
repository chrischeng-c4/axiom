# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "type"
# case = "XMLParserType__SetBillionLaughsAttackProtectionMaximumAmplification__max_factor_as_float_wrong"
# subject = "pyexpat.XMLParserType.SetBillionLaughsAttackProtectionMaximumAmplification(max_factor: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyexpat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pyexpat.XMLParserType.SetBillionLaughsAttackProtectionMaximumAmplification(max_factor: float); call it with the wrong type.

typeshed contract: max_factor is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pyexpat import XMLParserType
obj = object.__new__(XMLParserType)
try:
    obj.SetBillionLaughsAttackProtectionMaximumAmplification("not_a_float")  # max_factor: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
