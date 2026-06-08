# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "public_type_name_metadata"
# subject = "weakref.WeakValueDictionary"
# kind = "semantic"
# xfail = "mamba weakref shim classes lack stable __name__/__qualname__/__module__ metadata (gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.WeakValueDictionary: public weakref classes expose stable __name__/__qualname__/__module__ dotted-name metadata"""
import weakref


# Public type objects expose clean dotted-name metadata.
for name in (
    "ReferenceType", "ProxyType", "CallableProxyType", "WeakMethod",
    "WeakSet", "WeakKeyDictionary", "WeakValueDictionary",
):
    obj = getattr(weakref, name)
    assert obj.__name__ == name, f"{name}.__name__ -> {obj.__name__!r}"
    assert obj.__qualname__ == name, f"{name}.__qualname__ -> {obj.__qualname__!r}"
    # WeakSet lives in the _weakrefset helper module, the rest in weakref.
    if name != "WeakSet":
        assert obj.__module__ == "weakref", f"{name}.__module__ -> {obj.__module__!r}"

print("public_type_name_metadata OK")
