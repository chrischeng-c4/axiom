# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "behavior"
# case = "vars_user_instance_dict"
# subject = "builtins.vars(user instance)"
# kind = "semantic"
# xfail = "vars(obj) must return the identical __dict__; mamba currently returns an equal proxy"
# mem_carveout = ""
# source = "Lib/test/test_builtin.py"
# status = "filled"
# ///
# mamba-xfail: vars(obj) must return the identical __dict__; mamba currently returns an equal proxy
"""builtins.vars returns the instance dictionary for ordinary user objects."""

class _W:
    def __init__(self) -> None:
        self.answer = 42


obj = _W()
assert vars(obj) == {"answer": 42}
assert vars(obj) is obj.__dict__
print("vars_user_instance_dict OK")
