# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "uuid_class_is_callable"
# subject = "uuid.UUID"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.UUID: uuid_class_is_callable (surface)."""
import uuid

assert callable(uuid.UUID)
print("uuid_class_is_callable OK")
