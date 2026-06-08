# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "behavior"
# case = "proxy_bytes_and_dir_forwarding"
# subject = "weakref.proxy"
# kind = "semantic"
# xfail = "mamba proxy not protocol-transparent: __bytes__/dir forwarding diverges (refcount-only shim, gh #1466)"
# mem_carveout = ""
# source = "Lib/test/test_weakref.py"
# status = "filled"
# ///
"""weakref.proxy: bytes(proxy) forwards __bytes__ and dir(proxy) exposes the referent's methods"""
import weakref


# bytes() forwards __bytes__; dir() exposes the referent's methods.
class Stringy:
    def __str__(self):
        return "string"

    def __bytes__(self):
        return b"bytes"


stringy = Stringy()
p_conv = weakref.proxy(stringy)
assert "__bytes__" in dir(p_conv), "dir(proxy) exposes __bytes__"
assert bytes(p_conv) == b"bytes", f"bytes(proxy) -> {bytes(p_conv)!r}"

print("proxy_bytes_and_dir_forwarding OK")
