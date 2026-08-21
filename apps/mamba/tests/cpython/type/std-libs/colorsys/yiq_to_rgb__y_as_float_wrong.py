# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "colorsys"
# dimension = "type"
# case = "yiq_to_rgb__y_as_float_wrong"
# subject = "colorsys.yiq_to_rgb(y: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/colorsys.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: colorsys.yiq_to_rgb(y: float); call it with the wrong type.

typeshed contract: y is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from colorsys import yiq_to_rgb
try:
    yiq_to_rgb("not_a_float", 0.0, 0.0)  # y: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
