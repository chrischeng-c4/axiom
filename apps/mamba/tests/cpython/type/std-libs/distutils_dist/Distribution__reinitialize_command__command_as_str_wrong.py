# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_dist"
# dimension = "type"
# case = "Distribution__reinitialize_command__command_as_str_wrong"
# subject = "distutils.dist.Distribution.reinitialize_command(command: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed command"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/dist.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed command
# mamba-strict-type: TypeError
"""Type wall: distutils.dist.Distribution.reinitialize_command(command: str); call it with the wrong type.

typeshed contract: command is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.dist import Distribution
obj = object.__new__(Distribution)
try:
    obj.reinitialize_command(12345)  # command: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
