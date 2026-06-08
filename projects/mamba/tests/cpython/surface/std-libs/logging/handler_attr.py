# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "handler_attr"
# subject = "logging"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging: handler_attr (surface)."""
import logging

assert hasattr(logging, "Handler")
print("handler_attr OK")
