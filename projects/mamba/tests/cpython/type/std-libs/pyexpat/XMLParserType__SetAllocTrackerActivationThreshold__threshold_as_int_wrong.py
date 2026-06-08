# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pyexpat"
# dimension = "type"
# case = "XMLParserType__SetAllocTrackerActivationThreshold__threshold_as_int_wrong"
# subject = "pyexpat.XMLParserType.SetAllocTrackerActivationThreshold(threshold: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pyexpat.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pyexpat.XMLParserType.SetAllocTrackerActivationThreshold(threshold: int); call it with the wrong type.

typeshed contract: threshold is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pyexpat import XMLParserType
obj = object.__new__(XMLParserType)
try:
    obj.SetAllocTrackerActivationThreshold("not_an_int")  # threshold: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
