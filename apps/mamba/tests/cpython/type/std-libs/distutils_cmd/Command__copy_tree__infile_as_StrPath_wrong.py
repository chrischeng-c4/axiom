# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_cmd"
# dimension = "type"
# case = "Command__copy_tree__infile_as_StrPath_wrong"
# subject = "distutils.cmd.Command.copy_tree(infile: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/cmd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.cmd.Command.copy_tree(infile: StrPath); call it with the wrong type.

typeshed contract: infile is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.cmd import Command
obj = object.__new__(Command)
try:
    obj.copy_tree(_W(), "")  # infile: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
