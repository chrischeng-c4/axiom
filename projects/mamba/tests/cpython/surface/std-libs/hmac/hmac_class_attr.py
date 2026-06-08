# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "surface"
# case = "hmac_class_attr"
# subject = "hmac"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hmac: hmac_class_attr (surface)."""
import hmac

assert hasattr(hmac, "HMAC")
print("hmac_class_attr OK")
