# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "HTMLCalendar__formatyearpage__theyear_as_int_wrong"
# subject = "calendar.HTMLCalendar.formatyearpage(theyear: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.HTMLCalendar.formatyearpage(theyear: int); call it with the wrong type.

typeshed contract: theyear is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from calendar import HTMLCalendar
obj = object.__new__(HTMLCalendar)
try:
    obj.formatyearpage("not_an_int")  # theyear: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
