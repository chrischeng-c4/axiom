# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "dedent_is_callable"
# subject = "textwrap.dedent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap.dedent: dedent_is_callable (surface)."""
import textwrap

assert callable(textwrap.dedent)
print("dedent_is_callable OK")
