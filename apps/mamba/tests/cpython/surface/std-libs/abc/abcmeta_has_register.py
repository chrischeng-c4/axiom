# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "abc"
# dimension = "surface"
# case = "abcmeta_has_register"
# subject = "abc.ABCMeta"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""abc.ABCMeta: abcmeta_has_register (surface)."""
import abc

assert hasattr(abc.ABCMeta, "register")
print("abcmeta_has_register OK")
