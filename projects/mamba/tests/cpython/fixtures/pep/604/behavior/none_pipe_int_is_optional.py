# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "behavior"
# case = "none_pipe_int_is_optional"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = "`None | int` returns None on mamba (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: None | int yields a UnionType whose __args__ is (NoneType, int) and whose repr is 'None | int'"""
import types

opt = None | int
assert isinstance(opt, types.UnionType)
assert opt.__args__ == (type(None), int)
assert repr(opt) == "None | int"
# It still works as an isinstance check over both members.
assert isinstance(None, opt) is True
assert isinstance(5, opt) is True
assert isinstance("a", opt) is False

print("none_pipe_int_is_optional OK")
