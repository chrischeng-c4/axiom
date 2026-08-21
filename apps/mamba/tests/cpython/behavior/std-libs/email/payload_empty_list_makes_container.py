# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "payload_empty_list_makes_container"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: set_payload([]) makes the message an (empty) container: get_payload() == []"""
from email.message import Message

container = Message()
container.set_payload([])
assert container.get_payload() == [], container.get_payload()

print("payload_empty_list_makes_container OK")
