# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "behavior"
# case = "generator_is_iterator_iterable"
# subject = "collections.abc.Generator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Generator: a generator object is an instance of Generator, Iterator, and Iterable"""
import collections.abc as abc


def gen():
    yield 1
    yield 2


g = gen()
assert isinstance(g, abc.Generator), "generator is Generator"
assert isinstance(g, abc.Iterator), "generator is Iterator"
assert isinstance(g, abc.Iterable), "generator is Iterable"
print("generator_is_iterator_iterable OK")
