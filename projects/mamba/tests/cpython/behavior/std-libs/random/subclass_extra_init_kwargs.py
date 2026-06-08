# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "subclass_extra_init_kwargs"
# subject = "random.Random"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random: a Random subclass may add extra __init__ keyword arguments and still construct: WithKwarg(newarg=1) succeeds when it calls Random.__init__(self)"""
import random

class WithKwarg(random.Random):
    def __init__(self, newarg=None):
        random.Random.__init__(self)

WithKwarg(newarg=1)

print("subclass_extra_init_kwargs OK")
