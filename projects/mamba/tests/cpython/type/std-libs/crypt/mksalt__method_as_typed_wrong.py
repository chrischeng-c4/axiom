# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crypt"
# dimension = "type"
# case = "mksalt__method_as_typed_wrong"
# subject = "crypt.mksalt(method: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/crypt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: crypt.mksalt(method: typed); call it with the wrong type.

typeshed contract: method is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from crypt import mksalt
try:
    mksalt(_W())  # method: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
