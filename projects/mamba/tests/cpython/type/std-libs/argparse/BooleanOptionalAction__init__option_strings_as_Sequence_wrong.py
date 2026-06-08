# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "type"
# case = "BooleanOptionalAction__init__option_strings_as_Sequence_wrong"
# subject = "argparse.BooleanOptionalAction.__init__(option_strings: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed option_strings"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/argparse.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed option_strings
# mamba-strict-type: TypeError
"""Type wall: argparse.BooleanOptionalAction.__init__(option_strings: Sequence); call it with the wrong type.

typeshed contract: option_strings is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from argparse import BooleanOptionalAction
try:
    BooleanOptionalAction(_W(), "")  # option_strings: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
