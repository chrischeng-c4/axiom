# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selectors"
# dimension = "surface"
# case = "abcmeta_attr"
# subject = "selectors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""selectors: abcmeta_attr (surface)."""
import selectors

assert hasattr(selectors, "ABCMeta")
print("abcmeta_attr OK")
