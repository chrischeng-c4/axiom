# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "textwrapper_has_wrap"
# subject = "textwrap.TextWrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.TextWrapper: textwrapper_has_wrap (surface)."""
import textwrap

assert hasattr(textwrap.TextWrapper, "wrap")
print("textwrapper_has_wrap OK")
