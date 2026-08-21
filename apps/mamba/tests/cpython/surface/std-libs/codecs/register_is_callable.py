# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "surface"
# case = "register_is_callable"
# subject = "codecs.register"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""codecs.register: register_is_callable (surface)."""
import codecs

assert callable(codecs.register)
print("register_is_callable OK")
