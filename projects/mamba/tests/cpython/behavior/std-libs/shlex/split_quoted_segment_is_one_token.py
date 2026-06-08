# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shlex"
# dimension = "behavior"
# case = "split_quoted_segment_is_one_token"
# subject = "shlex.split"
# kind = "semantic"
# xfail = "mamba shlex.split does not process quotes (returns raw whitespace split); repo-memory project_mamba_stdlib_stub_audit_2026_05_26"
# mem_carveout = ""
# source = "Lib/test/test_shlex.py"
# status = "filled"
# ///
"""shlex.split: a double-quoted segment is collapsed into a single token with the quotes stripped, e.g. split('"hello world" foo') == ['hello world', 'foo']"""
import shlex

assert shlex.split('"hello world" foo') == ["hello world", "foo"], 'double-quoted segment is one token'
assert shlex.split("'a b' c") == ["a b", "c"], "single-quoted segment is one token"
print("split_quoted_segment_is_one_token OK")
