# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__format_option_strings__option_as_Option_wrong"
# subject = "optparse.HelpFormatter.format_option_strings(option: Option)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.format_option_strings(option: Option); call it with the wrong type.

typeshed contract: option is Option. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.format_option_strings(_W())  # option: Option <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
