# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_command_build_py"
# dimension = "type"
# case = "build_py__get_outputs__include_bytecode_as_typed_wrong"
# subject = "distutils.command.build_py.build_py.get_outputs(include_bytecode: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/command/build_py.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.command.build_py.build_py.get_outputs(include_bytecode: typed); call it with the wrong type.

typeshed contract: include_bytecode is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.command.build_py import build_py
obj = object.__new__(build_py)
try:
    obj.get_outputs(_W())  # include_bytecode: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
