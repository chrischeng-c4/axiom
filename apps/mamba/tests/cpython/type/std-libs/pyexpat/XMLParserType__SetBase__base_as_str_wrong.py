# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "type"
# case = "XMLParserType__SetBase__base_as_str_wrong"
# subject = "pyexpat.XMLParserType.SetBase(base: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyexpat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pyexpat.XMLParserType.SetBase(base: str); call it with the wrong type.

typeshed contract: base is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pyexpat import XMLParserType
obj = object.__new__(XMLParserType)
try:
    obj.SetBase(12345)  # base: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
