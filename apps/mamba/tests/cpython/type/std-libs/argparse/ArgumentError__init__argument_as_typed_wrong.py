# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "type"
# case = "ArgumentError__init__argument_as_typed_wrong"
# subject = "argparse.ArgumentError.__init__(argument: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/argparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: argparse.ArgumentError.__init__(argument: typed); call it with the wrong type.

typeshed contract: argument is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from argparse import ArgumentError
try:
    ArgumentError(_W(), "")  # argument: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
