# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_codecs"
# dimension = "type"
# case = "charmap_build__map_as_str_wrong"
# subject = "_codecs.charmap_build(map: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _codecs.charmap_build(map: str); call it with the wrong type.

typeshed contract: map is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _codecs import charmap_build
try:
    charmap_build(12345)  # map: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
