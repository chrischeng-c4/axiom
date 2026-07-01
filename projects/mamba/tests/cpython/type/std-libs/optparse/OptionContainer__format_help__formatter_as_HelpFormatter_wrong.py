# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionContainer__format_help__formatter_as_HelpFormatter_wrong"
# subject = "optparse.OptionContainer.format_help(formatter: HelpFormatter)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionContainer.format_help(formatter: HelpFormatter); call it with the wrong type.

typeshed contract: formatter is HelpFormatter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionContainer
obj = object.__new__(OptionContainer)
try:
    obj.format_help(_W())  # formatter: HelpFormatter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
