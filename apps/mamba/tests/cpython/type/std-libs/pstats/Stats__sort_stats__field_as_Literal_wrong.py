# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "type"
# case = "Stats__sort_stats__field_as_Literal_wrong"
# subject = "pstats.Stats.sort_stats(field: Literal)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed field"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pstats.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed field
# mamba-strict-type: TypeError
"""Type wall: pstats.Stats.sort_stats(field: Literal); call it with the wrong type.

typeshed contract: field is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pstats import Stats
obj = object.__new__(Stats)
try:
    obj.sort_stats(_W())  # field: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
