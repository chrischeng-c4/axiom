# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "string_parsing_tolerates_forms"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: the constructor accepts brace, urn:uuid:, dash-free, and upper-case spellings of one UUID, all equal to the canonical form"""
import uuid

canon = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert uuid.UUID("{12345678-1234-5678-1234-567812345678}") == canon, "brace form"
assert uuid.UUID("urn:uuid:12345678-1234-5678-1234-567812345678") == canon, "urn form"
assert uuid.UUID("12345678123456781234567812345678") == canon, "no-dash form"
assert uuid.UUID("12345678-1234-5678-1234-567812345678".upper()) == canon, "upper case"
print("string_parsing_tolerates_forms OK")
