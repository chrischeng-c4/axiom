# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "type"
# case = "uname_result____new____system_as_str_wrong"
# subject = "platform.uname_result.__new__(system: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/platform.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: platform.uname_result.__new__(system: str); call it with the wrong type.

typeshed contract: system is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from platform import uname_result
obj = object.__new__(uname_result)
try:
    obj.__new__(12345, "", "", "", "")  # system: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
