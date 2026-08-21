# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "isdatadescriptor_is_callable"
# subject = "inspect.isdatadescriptor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isdatadescriptor: isdatadescriptor_is_callable (surface)."""
import inspect

assert callable(inspect.isdatadescriptor)
print("isdatadescriptor_is_callable OK")
