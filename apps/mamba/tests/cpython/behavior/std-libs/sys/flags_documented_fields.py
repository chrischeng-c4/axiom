# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "flags_documented_fields"
# subject = "sys.flags"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.flags: every documented sys.flags field exists with the right type (bool for dev_mode/safe_path, int otherwise) and utf8_mode is 0/1/2"""
import sys

flag_names = (
    "debug", "inspect", "interactive", "optimize", "dont_write_bytecode",
    "no_user_site", "no_site", "ignore_environment", "verbose",
    "bytes_warning", "quiet", "hash_randomization", "isolated",
    "dev_mode", "utf8_mode", "warn_default_encoding", "safe_path",
    "int_max_str_digits",
)
for name in flag_names:
    assert hasattr(sys.flags, name), f"sys.flags missing {name}"
    expected = bool if name in ("dev_mode", "safe_path") else int
    assert type(getattr(sys.flags, name)) is expected, \
        f"flag {name} type = {type(getattr(sys.flags, name))!r}"
assert sys.flags.utf8_mode in (0, 1, 2), f"utf8_mode = {sys.flags.utf8_mode!r}"
print("flags_documented_fields OK")
