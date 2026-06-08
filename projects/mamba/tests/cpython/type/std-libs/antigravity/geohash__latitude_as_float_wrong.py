# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "antigravity"
# dimension = "type"
# case = "geohash__latitude_as_float_wrong"
# subject = "antigravity.geohash(latitude: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/antigravity.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: antigravity.geohash(latitude: float); call it with the wrong type.

typeshed contract: latitude is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from antigravity import geohash
try:
    geohash("not_a_float", 0.0, None)  # latitude: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
