# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_cmd"
# dimension = "type"
# case = "Command__ensure_string_list__option_as_str_wrong"
# subject = "distutils.cmd.Command.ensure_string_list(option: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.cmd.Command.ensure_string_list(option: str); call it with the wrong type.

typeshed contract: option is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.cmd import Command
obj = object.__new__(Command)
try:
    obj.ensure_string_list(12345)  # option: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
