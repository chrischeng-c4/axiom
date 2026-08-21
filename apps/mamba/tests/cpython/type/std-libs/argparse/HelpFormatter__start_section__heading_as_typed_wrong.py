# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "type"
# case = "HelpFormatter__start_section__heading_as_typed_wrong"
# subject = "argparse.HelpFormatter.start_section(heading: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/argparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: argparse.HelpFormatter.start_section(heading: typed); call it with the wrong type.

typeshed contract: heading is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from argparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.start_section(_W())  # heading: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
