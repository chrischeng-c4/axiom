# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "custom_action_call"
# subject = "argparse.Action"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.Action: a user-defined Action subclass has its __call__ invoked during parse_args, letting it write a derived value onto the Namespace"""
import argparse


class CollectAction(argparse.Action):
    def __call__(self, parser, namespace, values, option_string=None):
        setattr(namespace, self.dest, ["collected", values])


p = argparse.ArgumentParser()
p.add_argument("--spam", action=CollectAction)
ns = p.parse_args(["--spam", "eggs"])
assert ns.spam == ["collected", "eggs"], f"custom action result = {ns.spam!r}"
print("custom_action_call OK")
