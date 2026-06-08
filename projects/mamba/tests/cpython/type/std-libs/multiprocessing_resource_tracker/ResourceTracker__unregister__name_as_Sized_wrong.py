# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_resource_tracker"
# dimension = "type"
# case = "ResourceTracker__unregister__name_as_Sized_wrong"
# subject = "multiprocessing.resource_tracker.ResourceTracker.unregister(name: Sized)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/resource_tracker.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.resource_tracker.ResourceTracker.unregister(name: Sized); call it with the wrong type.

typeshed contract: name is Sized. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.resource_tracker import ResourceTracker
obj = object.__new__(ResourceTracker)
try:
    obj.unregister(_W(), "")  # name: Sized <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
