# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "surface"
# case = "nsig_is_int"
# subject = "signal.NSIG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.NSIG: nsig_is_int (surface)."""
import signal

assert type(signal.NSIG).__name__ == "int"
print("nsig_is_int OK")
