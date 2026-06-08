# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "excepthook_attr"
# subject = "threading"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading: excepthook_attr (surface)."""
import threading

assert hasattr(threading, "excepthook")
print("excepthook_attr OK")
