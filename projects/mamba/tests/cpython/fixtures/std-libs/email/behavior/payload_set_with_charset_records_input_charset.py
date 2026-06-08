# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "payload_set_with_charset_records_input_charset"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: set_payload(text, Charset('iso-8859-1')) records the input charset on the message (get_charset().input_charset == 'iso-8859-1')"""
from email.message import Message

from email.charset import Charset

ch = Message()
ch.set_payload("This is a string payload", Charset("iso-8859-1"))
assert ch.get_charset().input_charset == "iso-8859-1", ch.get_charset().input_charset

print("payload_set_with_charset_records_input_charset OK")
