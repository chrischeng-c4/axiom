# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "type"
# case = "concurrent_tee__iterable_as_Iterable_wrong"
# subject = "threading.concurrent_tee(iterable: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/threading.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: threading.concurrent_tee(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from threading import concurrent_tee
try:
    concurrent_tee(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
