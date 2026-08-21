# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string_templatelib"
# dimension = "type"
# case = "Template____add____other_as_Template_wrong"
# subject = "string.templatelib.Template.__add__(other: Template)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string/templatelib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.templatelib.Template.__add__(other: Template); call it with the wrong type.

typeshed contract: other is Template. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from string.templatelib import Template
obj = object.__new__(Template)
try:
    obj.__add__(_W())  # other: Template <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
