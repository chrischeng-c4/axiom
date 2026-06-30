# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "fancy_getopt__options_as_list_wrong"
# subject = "distutils.fancy_getopt.fancy_getopt(options: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.fancy_getopt(options: list); call it with the wrong type.

typeshed contract: options is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.fancy_getopt import fancy_getopt
try:
    fancy_getopt(12345, None, None, None)  # options: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
