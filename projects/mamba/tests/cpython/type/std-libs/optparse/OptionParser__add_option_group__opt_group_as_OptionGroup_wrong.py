# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "OptionParser__add_option_group__opt_group_as_OptionGroup_wrong"
# subject = "optparse.OptionParser.add_option_group(opt_group: OptionGroup)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed opt_group"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed opt_group
# mamba-strict-type: TypeError
"""Type wall: optparse.OptionParser.add_option_group(opt_group: OptionGroup); call it with the wrong type.

typeshed contract: opt_group is OptionGroup. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import OptionParser
obj = object.__new__(OptionParser)
try:
    obj.add_option_group(_W())  # opt_group: OptionGroup <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
