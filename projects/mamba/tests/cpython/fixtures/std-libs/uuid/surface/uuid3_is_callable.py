# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "uuid3_is_callable"
# subject = "uuid.uuid3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.uuid3: uuid3_is_callable (surface)."""
import uuid

assert callable(uuid.uuid3)
print("uuid3_is_callable OK")
