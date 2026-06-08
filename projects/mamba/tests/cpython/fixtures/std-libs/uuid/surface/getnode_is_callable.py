# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "surface"
# case = "getnode_is_callable"
# subject = "uuid.getnode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""uuid.getnode: getnode_is_callable (surface)."""
import uuid

assert callable(uuid.getnode)
print("getnode_is_callable OK")
