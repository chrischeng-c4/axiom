# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_cmd"
# dimension = "type"
# case = "Command__move_file__src_as_BytesPath_wrong"
# subject = "distutils.cmd.Command.move_file(src: BytesPath)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed src"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/cmd.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed src
# mamba-strict-type: TypeError
"""Type wall: distutils.cmd.Command.move_file(src: BytesPath); call it with the wrong type.

typeshed contract: src is BytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.cmd import Command
obj = object.__new__(Command)
try:
    obj.move_file(_W(), None)  # src: BytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
