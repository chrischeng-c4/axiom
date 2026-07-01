# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "calendar"
# dimension = "type"
# case = "formatstring__cols_as_Iterable_wrong"
# subject = "calendar.formatstring(cols: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/calendar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: calendar.formatstring(cols: Iterable); call it with the wrong type.

typeshed contract: cols is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from calendar import formatstring
try:
    formatstring(_W())  # cols: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
