# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "ExpandEnvironmentStrings__string_as_str_wrong"
# subject = "winreg.ExpandEnvironmentStrings(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.ExpandEnvironmentStrings(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from winreg import ExpandEnvironmentStrings
try:
    ExpandEnvironmentStrings(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
