# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "errors"
# case = "system_random_randbytes_rejects_negative"
# subject = "random.SystemRandom.randbytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.SystemRandom.randbytes: SystemRandom.randbytes rejects a negative count with ValueError, and choice([]) on an empty sequence raises IndexError"""
import random

gen = random.SystemRandom()

# randbytes rejects a negative count.
try:
    gen.randbytes(-1)
    raise AssertionError("expected ValueError for randbytes(-1)")
except ValueError:
    pass

# choice on an empty sequence raises IndexError.
try:
    gen.choice([])
    raise AssertionError("expected IndexError for empty choice")
except IndexError:
    pass

print("system_random_randbytes_rejects_negative OK")
