# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures__base"
# dimension = "type"
# case = "as_completed__fs_as_Iterable_wrong"
# subject = "concurrent.futures._base.as_completed(fs: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/_base.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures._base.as_completed(fs: Iterable); call it with the wrong type.

typeshed contract: fs is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures._base import as_completed
try:
    as_completed(_W())  # fs: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
