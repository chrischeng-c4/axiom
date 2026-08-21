# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "abstract_cm_default_enter_returns_self"
# subject = "contextlib.AbstractContextManager"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
"""contextlib.AbstractContextManager: the default __enter__ provided by AbstractContextManager returns self, so a concrete subclass that only defines __exit__ yields itself to `as`"""
import contextlib


class Concrete(contextlib.AbstractContextManager):
    def __exit__(self, *args):
        return None


obj = Concrete()
with obj as entered:
    assert entered is obj, "default __enter__ must return self"

print("abstract_cm_default_enter_returns_self OK")
