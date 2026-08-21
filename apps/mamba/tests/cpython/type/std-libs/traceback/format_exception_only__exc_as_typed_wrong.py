# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "format_exception_only__exc_as_typed_wrong"
# subject = "traceback.format_exception_only(exc: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exc"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exc
# mamba-strict-type: TypeError
"""Type wall: traceback.format_exception_only(exc: typed); call it with the wrong type.

typeshed contract: exc is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import format_exception_only
try:
    format_exception_only(_W())  # exc: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
