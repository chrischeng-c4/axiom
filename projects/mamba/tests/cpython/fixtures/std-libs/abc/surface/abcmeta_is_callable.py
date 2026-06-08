# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "abcmeta_is_callable"
# subject = "abc.ABCMeta"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abc.ABCMeta: abcmeta_is_callable (surface)."""
import abc

assert callable(abc.ABCMeta)
print("abcmeta_is_callable OK")
