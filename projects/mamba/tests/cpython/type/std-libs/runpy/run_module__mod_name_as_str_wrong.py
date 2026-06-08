# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "runpy"
# dimension = "type"
# case = "run_module__mod_name_as_str_wrong"
# subject = "runpy.run_module(mod_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/runpy.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: runpy.run_module(mod_name: str); call it with the wrong type.

typeshed contract: mod_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from runpy import run_module
try:
    run_module(12345)  # mod_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
