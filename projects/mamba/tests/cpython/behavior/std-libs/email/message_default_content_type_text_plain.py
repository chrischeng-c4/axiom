# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_default_content_type_text_plain"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: a bare Message with no Content-Type defaults to text/plain: get_content_type/maintype/subtype return text/plain, text, plain"""
from email.message import Message

empty = Message()
assert empty.get_content_type() == "text/plain", empty.get_content_type()
assert empty.get_content_maintype() == "text", empty.get_content_maintype()
assert empty.get_content_subtype() == "plain", empty.get_content_subtype()

print("message_default_content_type_text_plain OK")
