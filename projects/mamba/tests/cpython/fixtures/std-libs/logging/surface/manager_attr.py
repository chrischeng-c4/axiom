# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "surface"
# case = "manager_attr"
# subject = "logging"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""logging: manager_attr (surface)."""
import logging

assert hasattr(logging, "Manager")
print("manager_attr OK")
