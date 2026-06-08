# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__eval_print_amount__sel_as__Selector_wrong"
# subject = "pstats.Stats.eval_print_amount(sel: _Selector)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.eval_print_amount(sel: _Selector); call it with the wrong type.

typeshed contract: sel is _Selector. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.eval_print_amount(_W(), None, "")  # sel: _Selector <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
