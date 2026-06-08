# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "behavior"
# case = "message_get_filename_missing_vs_empty"
# subject = "email.message.Message"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_email.py"
# status = "filled"
# ///
"""email.message.Message: get_filename returns None when no filename is present, but returns '' for a value-less filename param"""
import email

assert email.message_from_string("From: foo\n").get_filename() is None, "no filename"
bogus = email.message_from_string("Content-Disposition: blarg; filename\n")
assert bogus.get_filename() == "", repr(bogus.get_filename())

print("message_get_filename_missing_vs_empty OK")
