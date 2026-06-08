# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "type"
# case = "translate__pat_as_str_wrong"
# subject = "fnmatch.translate(pat: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fnmatch.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fnmatch.translate(pat: str); call it with the wrong type.

typeshed contract: pat is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from fnmatch import translate
try:
    translate(12345)  # pat: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
