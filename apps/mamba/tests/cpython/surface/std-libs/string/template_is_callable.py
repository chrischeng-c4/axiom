# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "template_is_callable"
# subject = "string.Template"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""string.Template: template_is_callable (surface)."""
import string

assert callable(string.Template)
print("template_is_callable OK")
