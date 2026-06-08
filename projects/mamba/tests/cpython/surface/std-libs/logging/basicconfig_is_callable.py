# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "basicconfig_is_callable"
# subject = "logging.basicConfig"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging.basicConfig: basicconfig_is_callable (surface)."""
import logging

assert callable(logging.basicConfig)
print("basicconfig_is_callable OK")
