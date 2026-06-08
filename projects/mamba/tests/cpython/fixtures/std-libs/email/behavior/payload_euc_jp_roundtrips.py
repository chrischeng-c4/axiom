# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "payload_euc_jp_roundtrips"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_asian_codecs.py"
# status = "filled"
# ///
"""email.message.Message: an euc-jp string survives set_payload + get_payload(decode=True).decode(charset); the same text under utf-8 round-trips through utf-8 too"""
from email.message import Message

jcode = "euc-jp"
jhello = str(b"\xa5\xcf\xa5\xed\xa1\xbc\xa5\xef\xa1\xbc\xa5\xeb\xa5\xc9\xa1\xaa", jcode)
jp = Message()
jp.set_payload(jhello, jcode)
decoded = jp.get_payload(decode=True).decode(jp.get_content_charset())
assert decoded == jhello, "euc-jp payload round-trips"

# The same text stored under utf-8 round-trips through utf-8.
u8 = Message()
u8.set_payload(jhello, "utf-8")
assert u8.get_payload(decode=True).decode(u8.get_content_charset()) == jhello, "utf-8 round-trip"

print("payload_euc_jp_roundtrips OK")
