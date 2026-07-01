# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "type"
# case = "date____sub____value_as_Self_wrong"
# subject = "datetime.date.__sub__(value: Self)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/datetime.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: datetime.date.__sub__(value: Self); call it with the wrong type.

typeshed contract: value is Self. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from datetime import date
obj = date(2000, 1, 2)
try:
    obj.__sub__(_W())  # value: Self <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
