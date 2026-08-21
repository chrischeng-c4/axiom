# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "abstract_cm_structural_subclass"
# subject = "contextlib.AbstractContextManager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.AbstractContextManager: any class defining both __enter__ and __exit__ is a virtual subclass of AbstractContextManager via __subclasshook__; setting either to None opts the class back out"""
import contextlib


class FromScratch:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, tb):
        return None


# Structural subclassing: both dunders present -> virtual subclass.
assert issubclass(FromScratch, contextlib.AbstractContextManager)


# Setting __enter__ or __exit__ to None opts the class back out.
class NoEnter(FromScratch):
    __enter__ = None


class NoExit(FromScratch):
    __exit__ = None


assert not issubclass(NoEnter, contextlib.AbstractContextManager)
assert not issubclass(NoExit, contextlib.AbstractContextManager)

print("abstract_cm_structural_subclass OK")
