# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_delitem_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not protocol-transparent: __delitem__ forwarding diverges (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: del proxy[key] forwards __delitem__ to the referent"""
import weakref


# __delitem__ forwards through the proxy.
class Container:
    result = None

    def __delitem__(self, key):
        self.result = key


c = Container()
del weakref.proxy(c)[7]
assert c.result == 7, f"proxy del item -> {c.result!r}"

print("proxy_delitem_forwarding OK")
