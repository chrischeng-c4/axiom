# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "strsignal_keyword_descriptions"
# subject = "signal.strsignal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.strsignal: strsignal returns the OS description string carrying a stable keyword per signal (Interrupt for SIGINT, Terminated for SIGTERM, Hangup for SIGHUP) and accepts the raw int just like the enum member"""
import signal

# The exact text varies by platform (some append the number, e.g.
# "Interrupt: 2"), so assert on the stable English keyword, not the whole text.
sigint_desc = signal.strsignal(signal.SIGINT)
assert isinstance(sigint_desc, str), f"SIGINT desc type = {type(sigint_desc)!r}"
assert "Interrupt" in sigint_desc, f"SIGINT desc = {sigint_desc!r}"

assert "Terminated" in signal.strsignal(signal.SIGTERM), "SIGTERM keyword"
assert "Hangup" in signal.strsignal(signal.SIGHUP), "SIGHUP keyword"

# strsignal accepts the raw integer just as well as the enum member.
assert signal.strsignal(int(signal.SIGINT)) == sigint_desc, "int matches enum"

print("strsignal_keyword_descriptions OK")
