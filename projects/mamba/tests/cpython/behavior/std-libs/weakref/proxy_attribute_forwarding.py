# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_attribute_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: proxy forwards attribute reads transparently to the live referent"""
import weakref


class _Node:
    def __init__(self, val):
        self.val = val


n = _Node(5)
p = weakref.proxy(n)
assert p.val == 5, f"proxy attr forwards -> {p.val!r}"

print("proxy_attribute_forwarding OK")
