# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "pathlike_structural_subclass"
# subject = "os.PathLike"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.PathLike: a class defining __fspath__ is a virtual subclass/instance of os.PathLike; one lacking it is not; explicit subclassing routes through os.fspath; PathLike[bytes] is a GenericAlias"""
import os
import types


# A class defining __fspath__ is a virtual subclass of os.PathLike.
class Implicit:
    def __fspath__(self):
        return "/tmp/implicit"


assert issubclass(Implicit, os.PathLike), "structural subclass via __fspath__"
assert isinstance(Implicit(), os.PathLike), "structural instance via __fspath__"


# A class lacking __fspath__ is not a PathLike.
class NotPath:
    pass


assert not issubclass(NotPath, os.PathLike), "no __fspath__ -> not PathLike"
assert not isinstance(NotPath(), os.PathLike), "no __fspath__ instance"


# Explicit subclassing works and the protocol routes through os.fspath.
class Explicit(os.PathLike):
    def __fspath__(self):
        return "/tmp/explicit"


assert issubclass(Explicit, os.PathLike), "explicit subclass"
assert os.fspath(Explicit()) == "/tmp/explicit", "fspath on explicit subclass"

# os.PathLike supports PEP 585 subscription -> types.GenericAlias.
alias = os.PathLike[bytes]
assert isinstance(alias, types.GenericAlias), f"alias type = {type(alias)!r}"
print("pathlike_structural_subclass OK")
