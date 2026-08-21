# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "type"
# case = "lookup__name_as_typed_wrong"
# subject = "unicodedata.lookup(name: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unicodedata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unicodedata.lookup(name: typed); call it with the wrong type.

typeshed contract: name is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unicodedata import lookup
try:
    lookup(_W())  # name: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
