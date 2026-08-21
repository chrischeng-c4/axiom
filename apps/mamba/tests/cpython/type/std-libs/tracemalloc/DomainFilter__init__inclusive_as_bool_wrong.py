# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "DomainFilter__init__inclusive_as_bool_wrong"
# subject = "tracemalloc.DomainFilter.__init__(inclusive: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed inclusive"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed inclusive
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.DomainFilter.__init__(inclusive: bool); call it with the wrong type.

typeshed contract: inclusive is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tracemalloc import DomainFilter
try:
    DomainFilter("not_a_bool", 0)  # inclusive: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
