# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "surface"
# case = "gettempdir_is_callable"
# subject = "tempfile.gettempdir"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tempfile.gettempdir: gettempdir_is_callable (surface)."""
import tempfile

assert callable(tempfile.gettempdir)
print("gettempdir_is_callable OK")
