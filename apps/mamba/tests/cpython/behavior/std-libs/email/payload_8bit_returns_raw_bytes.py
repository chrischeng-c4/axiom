# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "payload_8bit_returns_raw_bytes"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: with Content-Transfer-Encoding 8bit, get_payload(decode=True) returns the raw bytes of the payload; a pre-encoded utf-8 byte payload comes back byte-identical"""
from email.message import Message

raw = "this is a br\xf6ken thing to do"
eight = Message()
eight["content-type"] = "text/plain"
eight["content-transfer-encoding"] = "8bit"
eight.set_payload(raw)
assert eight.get_payload(decode=True) == bytes(raw, "raw-unicode-escape"), "8bit raw bytes"

# A pre-encoded utf-8 byte payload comes back byte-identical under 8bit.
qb = "this is a qu\xe9stionable thing to do".encode("utf-8")
qm = Message()
qm["content-type"] = 'text/plain; charset="utf-8"'
qm["content-transfer-encoding"] = "8bit"
qm._payload = qb
assert qm.get_payload(decode=True) == qb, "raw utf-8 bytes preserved"

print("payload_8bit_returns_raw_bytes OK")
