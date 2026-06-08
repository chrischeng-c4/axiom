# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "type"
# case = "astuple__obj_as_DataclassInstance_wrong"
# subject = "dataclasses.astuple(obj: DataclassInstance)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed obj"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dataclasses.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed obj
# mamba-strict-type: TypeError
"""Type wall: dataclasses.astuple(obj: DataclassInstance); call it with the wrong type.

typeshed contract: obj is DataclassInstance. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dataclasses import astuple
try:
    astuple(_W())  # obj: DataclassInstance <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
