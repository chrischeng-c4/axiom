# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "type"
# case = "get_clock_info__name_as_Literal_wrong"
# subject = "time.get_clock_info(name: Literal)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/time.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: time.get_clock_info(name: Literal); call it with the wrong type.

typeshed contract: name is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from time import get_clock_info
try:
    get_clock_info(_W())  # name: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
