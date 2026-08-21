# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sre_parse"
# dimension = "type"
# case = "SubPattern____getitem____index_as_typed_wrong"
# subject = "sre_parse.SubPattern.__getitem__(index: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sre_parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sre_parse.SubPattern.__getitem__(index: typed); call it with the wrong type.

typeshed contract: index is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sre_parse import SubPattern
obj = object.__new__(SubPattern)
try:
    obj.__getitem__(_W())  # index: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
