# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "type"
# case = "Template__debug__flag_as_bool_wrong"
# subject = "pipes.Template.debug(flag: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed flag"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pipes.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed flag
# mamba-strict-type: TypeError
"""Type wall: pipes.Template.debug(flag: bool); call it with the wrong type.

typeshed contract: flag is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pipes import Template
obj = object.__new__(Template)
try:
    obj.debug("not_a_bool")  # flag: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
