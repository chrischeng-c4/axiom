# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "type"
# case = "new__key_as_typed_wrong"
# subject = "hmac.new(key: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/hmac.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: hmac.new(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from hmac import new
try:
    new(_W(), None, None)  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
