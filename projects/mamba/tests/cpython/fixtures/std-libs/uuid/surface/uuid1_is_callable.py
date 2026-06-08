# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "uuid1_is_callable"
# subject = "uuid.uuid1"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.uuid1: uuid1_is_callable (surface)."""
import uuid

assert callable(uuid.uuid1)
print("uuid1_is_callable OK")
