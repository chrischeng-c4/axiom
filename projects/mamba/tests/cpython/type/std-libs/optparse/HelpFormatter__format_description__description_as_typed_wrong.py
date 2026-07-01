# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "optparse"
# dimension = "type"
# case = "HelpFormatter__format_description__description_as_typed_wrong"
# subject = "optparse.HelpFormatter.format_description(description: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/optparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: optparse.HelpFormatter.format_description(description: typed); call it with the wrong type.

typeshed contract: description is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from optparse import HelpFormatter
obj = object.__new__(HelpFormatter)
try:
    obj.format_description(_W())  # description: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
