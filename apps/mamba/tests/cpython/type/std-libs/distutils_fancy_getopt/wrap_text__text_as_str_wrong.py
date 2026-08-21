# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_fancy_getopt"
# dimension = "type"
# case = "wrap_text__text_as_str_wrong"
# subject = "distutils.fancy_getopt.wrap_text(text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/fancy_getopt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.fancy_getopt.wrap_text(text: str); call it with the wrong type.

typeshed contract: text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.fancy_getopt import wrap_text
try:
    wrap_text(12345, 0)  # text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
