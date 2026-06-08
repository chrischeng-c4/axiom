# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "uuid5_is_callable"
# subject = "uuid.uuid5"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.uuid5: uuid5_is_callable (surface)."""
import uuid

assert callable(uuid.uuid5)
print("uuid5_is_callable OK")
