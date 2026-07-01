# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "type"
# case = "detect_encoding__b_as_typed_wrong"
# subject = "json.detect_encoding(b: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: json.detect_encoding(b: typed); call it with the wrong type.

typeshed contract: b is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from json import detect_encoding
try:
    detect_encoding(_W())  # b: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
