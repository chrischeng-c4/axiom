# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "type"
# case = "XMLParserType__SetReparseDeferralEnabled__enabled_as_bool_wrong"
# subject = "pyexpat.XMLParserType.SetReparseDeferralEnabled(enabled: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enabled"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyexpat.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enabled
# mamba-strict-type: TypeError
"""Type wall: pyexpat.XMLParserType.SetReparseDeferralEnabled(enabled: bool); call it with the wrong type.

typeshed contract: enabled is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pyexpat import XMLParserType
obj = object.__new__(XMLParserType)
try:
    obj.SetReparseDeferralEnabled("not_a_bool")  # enabled: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
