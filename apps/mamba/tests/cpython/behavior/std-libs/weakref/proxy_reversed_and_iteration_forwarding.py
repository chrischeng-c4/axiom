# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_reversed_and_iteration_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not protocol-transparent: __reversed__/__iter__ forwarding diverges (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: reversed(proxy) forwards __reversed__ and a proxy over an iterator can drive a for-loop"""
import weakref


# reversed() forwards __reversed__.
class Reversible:
    def __len__(self):
        return 3

    def __reversed__(self):
        return iter("cba")


rev = Reversible()
assert "".join(reversed(weakref.proxy(rev))) == "cba", "reversed(proxy)"


# A proxy whose referent is itself an iterator can drive a for-loop.
def gen():
    yield from [4, 5, 6]


it = gen()


class Iterates:
    def __iter__(self):
        return weakref.proxy(it)


assert list(Iterates()) == [4, 5, 6], "iterate through a proxied iterator"


# Proxying a non-iterator and using it for iteration raises TypeError.
not_iter = lambda: 0


class BadIter:
    def __iter__(self):
        return weakref.proxy(not_iter)


_raised = False
try:
    list(BadIter())
except TypeError:
    _raised = True
assert _raised, "non-iterator proxy raises TypeError on next()"

print("proxy_reversed_and_iteration_forwarding OK")
