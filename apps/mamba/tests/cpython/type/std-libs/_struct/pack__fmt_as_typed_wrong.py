# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "pack__fmt_as_typed_wrong"
# subject = "_struct.pack(fmt: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.pack(fmt: typed); call it with the wrong type.

typeshed contract: fmt is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import pack
try:
    pack(_W())  # fmt: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
