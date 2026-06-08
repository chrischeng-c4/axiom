# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "surface"
# case = "import_textwrap"
# subject = "textwrap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""textwrap: import_textwrap (surface)."""
import textwrap

assert hasattr(textwrap, "wrap")
print("import_textwrap OK")
