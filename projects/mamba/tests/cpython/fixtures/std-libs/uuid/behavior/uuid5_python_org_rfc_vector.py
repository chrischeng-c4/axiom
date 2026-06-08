# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid5_python_org_rfc_vector"
# subject = "uuid.uuid5"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid5: uuid5(NAMESPACE_DNS, 'python.org') equals the known SHA-1 vector '886313e1-3b8a-5372-9b90-0c9aee199e5d'"""
import uuid

known = uuid.uuid5(uuid.NAMESPACE_DNS, "python.org")
assert str(known) == "886313e1-3b8a-5372-9b90-0c9aee199e5d", \
    f"uuid5 known value = {str(known)!r}"
assert known.version == 5, f"uuid5 version = {known.version!r}"
print("uuid5_python_org_rfc_vector OK")
