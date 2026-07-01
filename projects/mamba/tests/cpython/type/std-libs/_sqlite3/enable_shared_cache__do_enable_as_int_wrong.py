# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_sqlite3"
# dimension = "type"
# case = "enable_shared_cache__do_enable_as_int_wrong"
# subject = "_sqlite3.enable_shared_cache(do_enable: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_sqlite3.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _sqlite3.enable_shared_cache(do_enable: int); call it with the wrong type.

typeshed contract: do_enable is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _sqlite3 import enable_shared_cache
try:
    enable_shared_cache("not_an_int")  # do_enable: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
