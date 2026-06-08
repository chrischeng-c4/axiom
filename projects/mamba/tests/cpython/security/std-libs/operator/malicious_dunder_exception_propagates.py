# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "security"
# case = "malicious_dunder_exception_propagates"
# subject = "operator.attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: attrgetter, itemgetter and methodcaller are thin delegators: an exception raised inside an attacker-controlled __getattr__/__getitem__ propagates unchanged and is never swallowed or remapped"""
import operator


class HostileAttr:
    def __getattr__(self, name):
        raise SyntaxError("attacker controls __getattr__")


class HostileItem:
    def __getitem__(self, key):
        raise SyntaxError("attacker controls __getitem__")


# attrgetter delegates to __getattribute__/__getattr__: the hostile exception
# escapes unchanged rather than being caught and turned into AttributeError.
_raised = False
try:
    operator.attrgetter("anything")(HostileAttr())
except SyntaxError:
    _raised = True
assert _raised, "attrgetter must propagate __getattr__ exception unchanged"

# itemgetter delegates to __getitem__: same contract.
_raised = False
try:
    operator.itemgetter(42)(HostileItem())
except SyntaxError:
    _raised = True
assert _raised, "itemgetter must propagate __getitem__ exception unchanged"

print("malicious_dunder_exception_propagates OK")
