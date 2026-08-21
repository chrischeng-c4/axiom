# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "errors"
# case = "malformed_header_collects_defect_no_raise"
# subject = "email.message_from_string"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message_from_string: parsing a header line without a colon under the default policy does NOT raise; the defect is collected in msg.defects (non-empty) instead"""
from email import message_from_string

# A header line with no colon is malformed. Under the default policy the parser
# does NOT raise; it records the defect on msg.defects instead.
msg = message_from_string("not_a_valid_header without colon\n")
assert len(msg.defects) >= 1, f"expected a collected defect, got {msg.defects!r}"

print("malformed_header_collects_defect_no_raise OK")
