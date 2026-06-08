# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "custom_action_constructed_at_add_time"
# subject = "argparse.Action"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Action: a user-defined Action subclass is instantiated at add_argument time with the resolved constructor keywords (dest/const/default), before any parse"""
import argparse


class Probe(Exception):
    pass


class ProbingAction(argparse.Action):
    def __init__(self, option_strings, dest, const=None, default=None, **kwargs):
        if dest == "spam" and const == 99 and default == 7:
            raise Probe()
        super().__init__(option_strings, dest, **kwargs)

    def __call__(self, *args, **kwargs):
        pass


parser = argparse.ArgumentParser()
_raised = False
try:
    parser.add_argument("--spam", action=ProbingAction, const=99, default=7)
except Probe:
    _raised = True
assert _raised, "action class instantiated at add_argument time"
print("custom_action_constructed_at_add_time OK")
