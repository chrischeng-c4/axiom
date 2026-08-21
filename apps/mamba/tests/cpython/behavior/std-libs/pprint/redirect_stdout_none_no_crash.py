# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "redirect_stdout_none_no_crash"
# subject = "pprint.pprint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pprint: with contextlib.redirect_stdout(None) both pprint() and PrettyPrinter().pprint() resolve the stream lazily and do not crash"""
import contextlib
import pprint

# The stream defaults to the *current* sys.stdout, resolved lazily at call
# time. Redirecting stdout to None must not crash either entry point.
with contextlib.redirect_stdout(None):
    pprint.pprint("redirected")
    pprint.PrettyPrinter().pprint("redirected")
print("redirect_stdout_none_no_crash OK")
