# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "template_present"
# subject = "tempfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile: template_present (surface)."""
import tempfile

assert hasattr(tempfile, "template")
print("template_present OK")
