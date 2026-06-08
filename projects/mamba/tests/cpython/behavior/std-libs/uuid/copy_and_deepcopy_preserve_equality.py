# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "copy_and_deepcopy_preserve_equality"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: copy.copy and copy.deepcopy of a UUID both compare equal to the original"""
import copy
import uuid

canon = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert copy.copy(canon) == canon, "copy equal"
assert copy.deepcopy(canon) == canon, "deepcopy equal"
print("copy_and_deepcopy_preserve_equality OK")
