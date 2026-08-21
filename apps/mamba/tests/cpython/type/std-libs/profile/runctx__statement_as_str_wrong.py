# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profile"
# dimension = "type"
# case = "runctx__statement_as_str_wrong"
# subject = "profile.runctx(statement: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/profile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: profile.runctx(statement: str); call it with the wrong type.

typeshed contract: statement is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from profile import runctx
try:
    runctx(12345, None, None)  # statement: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
