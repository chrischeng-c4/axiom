# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "class_object_is_weakreferenceable"
# subject = "weakref.ref"
# kind = "semantic"
# xfail = "mamba refcount-only: class-object ref does not expire on collection (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: a class object is weak-referenceable; the ref dies once the class is deleted and collected"""
import gc
import weakref


class Throwaway:
    pass


rc = weakref.ref(Throwaway)
assert rc() is Throwaway, "class ref alive"
Throwaway = None
gc.collect()
assert rc() is None, "class ref dead after class deleted"

print("class_object_is_weakreferenceable OK")
