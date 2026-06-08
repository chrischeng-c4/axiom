# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "select"
# dimension = "type"
# case = "select__rlist_as_Iterable_wrong"
# subject = "select.select(rlist: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/select.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: select.select(rlist: Iterable); call it with the wrong type.

typeshed contract: rlist is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from select import select
try:
    select(_W(), None, None)  # rlist: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
