# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "type"
# case = "HtmlDiff__init__tabsize_as_int_wrong"
# subject = "difflib.HtmlDiff.__init__(tabsize: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/difflib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: difflib.HtmlDiff.__init__(tabsize: int); call it with the wrong type.

typeshed contract: tabsize is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from difflib import HtmlDiff
try:
    HtmlDiff("not_an_int")  # tabsize: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
