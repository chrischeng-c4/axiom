# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_cmd"
# dimension = "type"
# case = "Command__set_undefined_options__src_cmd_as_str_wrong"
# subject = "distutils.cmd.Command.set_undefined_options(src_cmd: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed src_cmd"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/cmd.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed src_cmd
# mamba-strict-type: TypeError
"""Type wall: distutils.cmd.Command.set_undefined_options(src_cmd: str); call it with the wrong type.

typeshed contract: src_cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.cmd import Command
obj = object.__new__(Command)
try:
    obj.set_undefined_options(12345)  # src_cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
