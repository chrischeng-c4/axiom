# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "verify____call____enumeration_as__EnumerationT_wrong"
# subject = "enum.verify.__call__(enumeration: _EnumerationT)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enumeration"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enumeration
# mamba-strict-type: TypeError
"""Type wall: enum.verify.__call__(enumeration: _EnumerationT); call it with the wrong type.

typeshed contract: enumeration is _EnumerationT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import verify
obj = object.__new__(verify)
try:
    obj.__call__(_W())  # enumeration: _EnumerationT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
