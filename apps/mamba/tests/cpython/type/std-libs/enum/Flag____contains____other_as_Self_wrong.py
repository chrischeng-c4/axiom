# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "Flag____contains____other_as_Self_wrong"
# subject = "enum.Flag.__contains__(other: Self)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.Flag.__contains__(other: Self); call it with the wrong type.

typeshed contract: other is Self. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import Flag
obj = object.__new__(Flag)
try:
    obj.__contains__(_W())  # other: Self <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
