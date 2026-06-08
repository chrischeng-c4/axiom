# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "get_ident_is_callable"
# subject = "threading.get_ident"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.get_ident: get_ident_is_callable (surface)."""
import threading

assert callable(threading.get_ident)
print("get_ident_is_callable OK")
