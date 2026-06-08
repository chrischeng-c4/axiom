# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "nturl2path"
# dimension = "type"
# case = "pathname2url__p_as_str_wrong"
# subject = "nturl2path.pathname2url(p: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/nturl2path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: nturl2path.pathname2url(p: str); call it with the wrong type.

typeshed contract: p is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from nturl2path import pathname2url
try:
    pathname2url(12345)  # p: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
