# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "type"
# case = "get_terminal_size__fallback_as_tuple_wrong"
# subject = "shutil.get_terminal_size(fallback: tuple)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed fallback"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shutil.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed fallback
# mamba-strict-type: TypeError
"""Type wall: shutil.get_terminal_size(fallback: tuple); call it with the wrong type.

typeshed contract: fallback is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from shutil import get_terminal_size
try:
    get_terminal_size(12345)  # fallback: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
