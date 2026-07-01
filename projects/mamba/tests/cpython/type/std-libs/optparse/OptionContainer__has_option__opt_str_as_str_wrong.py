# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__has_option__opt_str_as_str_wrong"
# subject = "optparse.OptionContainer.has_option(opt_str: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.has_option(opt_str: str); call it with the wrong type.

typeshed contract: opt_str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.has_option(12345)  # opt_str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
