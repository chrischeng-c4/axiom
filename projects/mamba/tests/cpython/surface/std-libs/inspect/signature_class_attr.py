# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "signature_class_attr"
# subject = "inspect"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect: signature_class_attr (surface)."""
import inspect

assert hasattr(inspect, "Signature")
print("signature_class_attr OK")
