# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "uuid4_is_callable"
# subject = "uuid.uuid4"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.uuid4: uuid4_is_callable (surface)."""
import uuid

assert callable(uuid.uuid4)
print("uuid4_is_callable OK")
