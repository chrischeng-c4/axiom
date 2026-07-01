# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "isleap__year_as_int_wrong"
# subject = "calendar.isleap(year: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.isleap(year: int); call it with the wrong type.

typeshed contract: year is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import isleap
try:
    isleap("not_an_int")  # year: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
