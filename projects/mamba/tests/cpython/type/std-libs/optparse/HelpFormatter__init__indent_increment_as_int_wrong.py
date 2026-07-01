# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__init__indent_increment_as_int_wrong"
# subject = "optparse.HelpFormatter.__init__(indent_increment: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.__init__(indent_increment: int); call it with the wrong type.

typeshed contract: indent_increment is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import HelpFormatter
try:
    HelpFormatter("not_an_int", 0, None, None)  # indent_increment: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
