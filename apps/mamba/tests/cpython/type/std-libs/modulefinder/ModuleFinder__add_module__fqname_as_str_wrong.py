# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__add_module__fqname_as_str_wrong"
# subject = "modulefinder.ModuleFinder.add_module(fqname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.add_module(fqname: str); call it with the wrong type.

typeshed contract: fqname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.add_module(12345)  # fqname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
