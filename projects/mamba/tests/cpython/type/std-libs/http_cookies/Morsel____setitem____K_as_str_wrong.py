# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "type"
# case = "Morsel____setitem____K_as_str_wrong"
# subject = "http.cookies.Morsel.__setitem__(K: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookies.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookies.Morsel.__setitem__(K: str); call it with the wrong type.

typeshed contract: K is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookies import Morsel
obj = object.__new__(Morsel)
try:
    obj.__setitem__(12345, None)  # K: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
