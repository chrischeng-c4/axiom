# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "type"
# case = "ImpLoader__init__fullname_as_str_wrong"
# subject = "pkgutil.ImpLoader.__init__(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pkgutil.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pkgutil.ImpLoader.__init__(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pkgutil import ImpLoader
try:
    ImpLoader(12345, None, None, None)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
