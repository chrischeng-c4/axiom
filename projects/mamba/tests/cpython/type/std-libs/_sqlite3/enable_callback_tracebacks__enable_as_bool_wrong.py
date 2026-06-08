# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_sqlite3"
# dimension = "type"
# case = "enable_callback_tracebacks__enable_as_bool_wrong"
# subject = "_sqlite3.enable_callback_tracebacks(enable: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enable"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_sqlite3.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enable
# mamba-strict-type: TypeError
"""Type wall: _sqlite3.enable_callback_tracebacks(enable: bool); call it with the wrong type.

typeshed contract: enable is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _sqlite3 import enable_callback_tracebacks
try:
    enable_callback_tracebacks("not_a_bool")  # enable: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
