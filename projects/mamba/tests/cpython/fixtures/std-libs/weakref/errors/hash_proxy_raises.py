# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "errors"
# case = "hash_proxy_raises"
# subject = "weakref.proxy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: hash_proxy_raises (errors)."""
import weakref

class _Hashable:
    def __hash__(self):
        return 42

_h = _Hashable()
_p = weakref.proxy(_h)

_raised = False
try:
    hash(_p)
except TypeError:
    _raised = True
assert _raised, "hash_proxy_raises: expected TypeError"
print("hash_proxy_raises OK")
