# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_set_type_rewrites_token"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: set_type rewrites only the type token of an arbitrary named header, leaving its other params intact"""
from email.message import Message

st = Message()
st["X-Content-Type"] = "text/plain"
st.set_type("application/octet-stream", "X-Content-Type")
assert st["x-content-type"] == "application/octet-stream", st["x-content-type"]

print("message_set_type_rewrites_token OK")
