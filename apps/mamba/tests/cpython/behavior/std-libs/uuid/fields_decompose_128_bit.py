# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "fields_decompose_128_bit"
# subject = "uuid.UUID"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.UUID: UUID('12345678-1234-5678-1234-567812345678').fields decomposes into (time_low, time_mid, time_hi_version, clock_seq_hi_variant, clock_seq_low, node) matching each named accessor"""
import uuid

u = uuid.UUID("12345678-1234-5678-1234-567812345678")
assert u.fields == (0x12345678, 0x1234, 0x5678, 0x12, 0x34, 0x567812345678), \
    f"fields = {u.fields!r}"
assert u.time_low == 0x12345678, f"time_low = {u.time_low!r}"
assert u.time_mid == 0x1234, f"time_mid = {u.time_mid!r}"
assert u.time_hi_version == 0x5678, f"time_hi_version = {u.time_hi_version!r}"
assert u.clock_seq_hi_variant == 0x12, f"clock_seq_hi_variant = {u.clock_seq_hi_variant!r}"
assert u.clock_seq_low == 0x34, f"clock_seq_low = {u.clock_seq_low!r}"
assert u.node == 0x567812345678, f"node = {u.node!r}"
print("fields_decompose_128_bit OK")
