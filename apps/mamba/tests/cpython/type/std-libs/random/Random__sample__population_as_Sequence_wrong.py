# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "type"
# case = "Random__sample__population_as_Sequence_wrong"
# subject = "random.Random.sample(population: Sequence)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed population"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/random.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed population
# mamba-strict-type: TypeError
"""Type wall: random.Random.sample(population: Sequence); call it with the wrong type.

typeshed contract: population is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from random import Random
obj = object.__new__(Random)
try:
    obj.sample(_W(), 0)  # population: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
