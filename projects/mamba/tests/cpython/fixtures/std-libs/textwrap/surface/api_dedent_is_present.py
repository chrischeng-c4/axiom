# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "api_dedent_is_present"
# subject = "textwrap.dedent"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""textwrap.dedent: api_dedent_is_present (surface)."""
import textwrap

assert hasattr(textwrap, "dedent")
print("api_dedent_is_present OK")
