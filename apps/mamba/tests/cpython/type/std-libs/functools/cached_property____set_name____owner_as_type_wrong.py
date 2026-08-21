# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "type"
# case = "cached_property____set_name____owner_as_type_wrong"
# subject = "functools.cached_property.__set_name__(owner: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed owner"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/functools.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed owner
# mamba-strict-type: TypeError
"""Type wall: functools.cached_property.__set_name__(owner: type); call it with the wrong type.

typeshed contract: owner is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from functools import cached_property
obj = object.__new__(cached_property)
try:
    obj.__set_name__(_W(), "")  # owner: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
