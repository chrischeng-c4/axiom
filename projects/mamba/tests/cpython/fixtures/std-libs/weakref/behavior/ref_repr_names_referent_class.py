# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "ref_repr_names_referent_class"
# subject = "weakref.ref"
# kind = "semantic"
# xfail = "mamba ref repr/str does not name the referent class (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.ref: ref() repr/str names the referent's class even when it overrides __getattr__ (gh-99184)"""
import weakref


# ref() repr names the referent's class even with a custom __getattr__
# that would otherwise hijack attribute access (gh-99184).
class MyConfig(dict):
    def __getattr__(self, key):
        return self[key]


cfg = MyConfig(offset=5)
r = weakref.ref(cfg)
assert "MyConfig" in repr(r), f"repr names class -> {repr(r)!r}"
assert "MyConfig" in str(r), f"str names class -> {str(r)!r}"

print("ref_repr_names_referent_class OK")
