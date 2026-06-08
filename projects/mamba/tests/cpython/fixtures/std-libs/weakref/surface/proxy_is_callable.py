# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "weakref"
# dimension = "surface"
# case = "proxy_is_callable"
# subject = "weakref.proxy"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""weakref.proxy: proxy_is_callable (surface)."""
import weakref

assert callable(weakref.proxy)
print("proxy_is_callable OK")
