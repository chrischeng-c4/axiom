# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Formatter__check_unused_args__used_args_as_set_wrong"
# subject = "string.Formatter.check_unused_args(used_args: set)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed used_args"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed used_args
# mamba-strict-type: TypeError
"""Type wall: string.Formatter.check_unused_args(used_args: set); call it with the wrong type.

typeshed contract: used_args is set. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from string import Formatter
obj = object.__new__(Formatter)
try:
    obj.check_unused_args(12345, None, None)  # used_args: set <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
