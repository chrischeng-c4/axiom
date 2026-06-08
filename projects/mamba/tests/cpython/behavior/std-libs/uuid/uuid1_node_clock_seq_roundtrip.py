# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "uuid1_node_clock_seq_roundtrip"
# subject = "uuid.uuid1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.uuid1: uuid1(node, clock_seq) with explicit values round-trips both fields, reports version 1, RFC 4122 variant, and a positive 60-bit time"""
import uuid

NODE = 93328246233727       # a valid 48-bit EUI-64 node
CLOCK_SEQ = 5317            # 14-bit clock sequence

u = uuid.uuid1(node=NODE, clock_seq=CLOCK_SEQ)
assert u.version == 1, f"version = {u.version!r}"
assert u.node == NODE, f"node = {u.node!r}"
assert u.clock_seq == CLOCK_SEQ, f"clock_seq = {u.clock_seq!r}"
assert u.variant == uuid.RFC_4122, f"variant = {u.variant!r}"
assert isinstance(u.time, int) and u.time > 0, f"time = {u.time!r}"
print("uuid1_node_clock_seq_roundtrip OK")
