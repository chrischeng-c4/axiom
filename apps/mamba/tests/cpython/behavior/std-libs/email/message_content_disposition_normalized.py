# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_content_disposition_normalized"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: get_content_disposition() lower-cases the disposition token while the raw header value keeps its spelling; add_header/replace_header and get_filename follow suit"""
from email.message import Message

disp = Message()
assert disp.get_content_disposition() is None, "no disposition yet"
disp.add_header("Content-Disposition", "attachment", filename="random.avi")
assert disp.get_content_disposition() == "attachment", disp.get_content_disposition()
assert disp.get_filename() == "random.avi", disp.get_filename()
disp.replace_header("Content-Disposition", "InlinE")
assert disp.get_content_disposition() == "inline", disp.get_content_disposition()
assert disp["content-disposition"] == "InlinE", disp["content-disposition"]

print("message_content_disposition_normalized OK")
