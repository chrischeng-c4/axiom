# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____le____value_as_date_wrong"
# subject = "datetime.date.__le__(value: date)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__le__(value: date); call it with the wrong type.

typeshed contract: value is date. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = object.__new__(date)
try:
    obj.__le__(_W())  # value: date <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
