# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "errors"
# case = "argument_error_str_is_message"
# subject = "argparse.ArgumentError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.ArgumentError: ArgumentError(None, msg) with no bound action stringifies to just the message text"""
import argparse

err = argparse.ArgumentError(None, "my error here")
assert str(err) == "my error here", f"argument_error str = {str(err)!r}"
print("argument_error_str_is_message OK")
