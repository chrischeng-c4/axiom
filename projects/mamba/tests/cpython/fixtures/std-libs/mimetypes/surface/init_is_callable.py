# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "surface"
# case = "init_is_callable"
# subject = "mimetypes.init"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.init: init_is_callable (surface)."""
import mimetypes

assert callable(mimetypes.init)
print("init_is_callable OK")
