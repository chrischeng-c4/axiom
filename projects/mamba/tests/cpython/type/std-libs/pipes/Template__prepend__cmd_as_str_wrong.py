# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pipes"
# dimension = "type"
# case = "Template__prepend__cmd_as_str_wrong"
# subject = "pipes.Template.prepend(cmd: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pipes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pipes.Template.prepend(cmd: str); call it with the wrong type.

typeshed contract: cmd is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pipes import Template
obj = object.__new__(Template)
try:
    obj.prepend(12345, "")  # cmd: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
