# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "type"
# case = "Action____call____parser_as_ArgumentParser_wrong"
# subject = "argparse.Action.__call__(parser: ArgumentParser)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/argparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: argparse.Action.__call__(parser: ArgumentParser); call it with the wrong type.

typeshed contract: parser is ArgumentParser. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from argparse import Action
obj = object.__new__(Action)
try:
    obj.__call__(_W(), None, None)  # parser: ArgumentParser <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
