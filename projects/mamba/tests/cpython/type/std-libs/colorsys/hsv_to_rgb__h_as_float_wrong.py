# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "type"
# case = "hsv_to_rgb__h_as_float_wrong"
# subject = "colorsys.hsv_to_rgb(h: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/colorsys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: colorsys.hsv_to_rgb(h: float); call it with the wrong type.

typeshed contract: h is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from colorsys import hsv_to_rgb
try:
    hsv_to_rgb("not_a_float", 0.0, 0.0)  # h: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
