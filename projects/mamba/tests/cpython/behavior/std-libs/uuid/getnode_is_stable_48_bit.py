# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "uuid"
# dimension = "behavior"
# case = "getnode_is_stable_48_bit"
# subject = "uuid.getnode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_uuid.py"
# status = "filled"
# ///
"""uuid.getnode: getnode() returns a stable 48-bit unsigned node id (repeat calls equal) that feeds uuid1().node within range"""
import uuid

node = uuid.getnode()
assert isinstance(node, int), f"node type = {type(node)!r}"
assert 0 < node < (1 << 48), f"node out of 48-bit range: {node!r}"
assert uuid.getnode() == node, "getnode not stable across calls"

u = uuid.uuid1()
assert 0 < u.node < (1 << 48), f"uuid1 node out of range: {u.node!r}"
print("getnode_is_stable_48_bit OK")
