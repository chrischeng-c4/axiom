# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "type"
# case = "quote_from_bytes__bs_as_typed_wrong"
# subject = "urllib.parse.quote_from_bytes(bs: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.parse.quote_from_bytes(bs: typed); call it with the wrong type.

typeshed contract: bs is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.parse import quote_from_bytes
try:
    quote_from_bytes(_W())  # bs: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
