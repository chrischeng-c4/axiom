# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_cmd"
# dimension = "type"
# case = "Command__reinitialize_command__command_as__CommandT_wrong"
# subject = "distutils.cmd.Command.reinitialize_command(command: _CommandT)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed command"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/cmd.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed command
# mamba-strict-type: TypeError
"""Type wall: distutils.cmd.Command.reinitialize_command(command: _CommandT); call it with the wrong type.

typeshed contract: command is _CommandT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.cmd import Command
obj = object.__new__(Command)
try:
    obj.reinitialize_command(_W())  # command: _CommandT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
