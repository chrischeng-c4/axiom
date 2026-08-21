# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "pow__base_as__SupportsPow3_wrong"
# subject = "builtins.pow(base: _SupportsPow3)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.pow(base: _SupportsPow3); call it with the wrong type.

typeshed contract: base is _SupportsPow3. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    pow(_W(), None, None)  # base: _SupportsPow3 <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
