# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_index_and_bool_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not protocol-transparent: __index__/__bool__ forwarding diverges (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: operator.index(proxy) forwards __index__ and bool(proxy) mirrors referent truthiness"""
import operator
import weakref


# operator.index() forwards __index__.
class Indexable:
    def __index__(self):
        return 10


idx = Indexable()
assert operator.index(weakref.proxy(idx)) == 10, "proxy __index__"


# bool() mirrors the referent's truthiness.
class EmptyList(list):
    pass


empty = EmptyList()
assert bool(weakref.proxy(empty)) is False, "empty proxy is falsey"
empty.append(1)
assert bool(weakref.proxy(empty)) is True, "non-empty proxy is truthy"

print("proxy_index_and_bool_forwarding OK")
