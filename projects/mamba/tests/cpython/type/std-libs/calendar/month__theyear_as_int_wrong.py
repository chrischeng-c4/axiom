# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "month__theyear_as_int_wrong"
# subject = "calendar.month(theyear: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.month(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import month
try:
    month("not_an_int", 0)  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
