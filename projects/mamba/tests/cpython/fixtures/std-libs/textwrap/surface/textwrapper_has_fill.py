# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "textwrapper_has_fill"
# subject = "textwrap.TextWrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.TextWrapper: textwrapper_has_fill (surface)."""
import textwrap

assert hasattr(textwrap.TextWrapper, "fill")
print("textwrapper_has_fill OK")
