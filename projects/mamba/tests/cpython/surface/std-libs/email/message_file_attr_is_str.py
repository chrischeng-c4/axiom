# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "message_file_attr_is_str"
# subject = "email.message.__file__"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""email.message.__file__: message_file_attr_is_str (surface)."""
import email.message

assert type(email.message.__file__).__name__ == "str"
print("message_file_attr_is_str OK")
