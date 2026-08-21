# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__get_print_list__sel_list_as_Iterable_wrong"
# subject = "pstats.Stats.get_print_list(sel_list: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.get_print_list(sel_list: Iterable); call it with the wrong type.

typeshed contract: sel_list is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.get_print_list(_W())  # sel_list: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
