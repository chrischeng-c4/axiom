# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Template__safe_substitute__mapping_as_Mapping_wrong"
# subject = "string.Template.safe_substitute(mapping: Mapping)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed mapping"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed mapping
# mamba-strict-type: TypeError
"""Type wall: string.Template.safe_substitute(mapping: Mapping); call it with the wrong type.

typeshed contract: mapping is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from string import Template
obj = object.__new__(Template)
try:
    obj.safe_substitute(_W())  # mapping: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
