# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "behavior"
# case = "error_class_hierarchy"
# subject = "pickle.PickleError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle.PickleError: PicklingError and UnpicklingError are both subclasses of pickle.PickleError"""
import pickle

assert issubclass(pickle.PicklingError, pickle.PickleError), "PicklingError <: PickleError"
assert issubclass(pickle.UnpicklingError, pickle.PickleError), "UnpicklingError <: PickleError"

print("error_class_hierarchy OK")
