# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "set_servername_callback_non_callable_raises"
# subject = "ssl.SSLContext.set_servername_callback"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.set_servername_callback: set_servername_callback rejects non-callables (an int, an empty str, a context object) with TypeError, but accepts None and a callable"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
for _bad in (4, "", _ctx):
    try:
        _ctx.set_servername_callback(_bad)
        raise AssertionError(f"set_servername_callback({_bad!r}) should raise")
    except TypeError:
        pass
# None and a real callable are accepted.
_ctx.set_servername_callback(None)
_ctx.set_servername_callback(lambda sock, name, context: None)

print("set_servername_callback_non_callable_raises OK")
