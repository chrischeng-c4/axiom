# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Formatter__vformat__format_string_as_str_wrong"
# subject = "string.Formatter.vformat(format_string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.Formatter.vformat(format_string: str); call it with the wrong type.

typeshed contract: format_string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from string import Formatter
obj = object.__new__(Formatter)
try:
    obj.vformat(12345, None, None)  # format_string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
