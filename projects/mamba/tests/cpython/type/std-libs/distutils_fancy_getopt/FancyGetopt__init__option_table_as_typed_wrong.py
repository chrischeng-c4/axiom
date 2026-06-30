# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "FancyGetopt__init__option_table_as_typed_wrong"
# subject = "distutils.fancy_getopt.FancyGetopt.__init__(option_table: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.FancyGetopt.__init__(option_table: typed); call it with the wrong type.

typeshed contract: option_table is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.fancy_getopt import FancyGetopt
try:
    FancyGetopt(_W())  # option_table: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
