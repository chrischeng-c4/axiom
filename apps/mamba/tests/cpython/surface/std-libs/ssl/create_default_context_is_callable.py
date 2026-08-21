# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "create_default_context_is_callable"
# subject = "ssl.create_default_context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl.create_default_context: create_default_context_is_callable (surface)."""
import ssl

assert callable(ssl.create_default_context)
print("create_default_context_is_callable OK")
