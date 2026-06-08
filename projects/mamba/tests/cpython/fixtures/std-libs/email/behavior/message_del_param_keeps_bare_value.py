# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_del_param_keeps_bare_value"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: del_param drops a single param (filename) but keeps the bare disposition value (attachment)"""
from email.message import Message

disp = Message()
disp.add_header("Content-Disposition", "attachment", filename="bud.gif")
disp.del_param("filename", "content-disposition")
assert disp["content-disposition"] == "attachment", disp["content-disposition"]

print("message_del_param_keeps_bare_value OK")
