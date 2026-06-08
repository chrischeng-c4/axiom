# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "behavior"
# case = "invalidate_caches_returns_none"
# subject = "importlib.invalidate_caches"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.invalidate_caches: invalidate_caches() runs without error and returns None"""
import importlib

assert importlib.invalidate_caches() is None, "invalidate_caches() returns None"
print("invalidate_caches_returns_none OK")
