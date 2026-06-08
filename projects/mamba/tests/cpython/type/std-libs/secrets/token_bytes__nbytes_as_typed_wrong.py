# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "type"
# case = "token_bytes__nbytes_as_typed_wrong"
# subject = "secrets.token_bytes(nbytes: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/secrets.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: secrets.token_bytes(nbytes: typed); call it with the wrong type.

typeshed contract: nbytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from secrets import token_bytes
try:
    token_bytes(_W())  # nbytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
