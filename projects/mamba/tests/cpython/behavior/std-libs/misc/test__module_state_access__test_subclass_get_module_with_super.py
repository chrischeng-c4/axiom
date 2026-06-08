# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "test__module_state_access__test_subclass_get_module_with_super"
# subject = "cpython.test_misc.Test_ModuleStateAccess.test_subclass_get_module_with_super"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import _thread
from collections import OrderedDict, deque
import contextlib
import importlib.machinery
import importlib.util
import json
import os
import pickle
import queue
import random
import subprocess
import sys
import textwrap
import threading
import time
import types
import warnings
import weakref
import operator
import _testinternalcapi
fullname = '_testmultiphase_meth_state_access'
origin = importlib.util.find_spec('_testmultiphase').origin
loader = importlib.machinery.ExtensionFileLoader(fullname, origin)
spec = importlib.util.spec_from_loader(fullname, loader)
module = importlib.util.module_from_spec(spec)
loader.exec_module(module)
self_module = module

class StateAccessType_Subclass(self_module.StateAccessType):

    def get_defining_module(self):
        return super().get_defining_module()
instance = StateAccessType_Subclass()
assert instance.get_defining_module() is self_module

print("Test_ModuleStateAccess::test_subclass_get_module_with_super: ok")
