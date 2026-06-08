# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_split_head_tail"
# subject = "os.path.split"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.split: os.path.split('/tmp/dir/file.txt') splits into ('/tmp/dir', 'file.txt')"""
import os.path

head, tail = os.path.split("/tmp/dir/file.txt")
assert head == "/tmp/dir", f"head = {head!r}"
assert tail == "file.txt", f"tail = {tail!r}"
print("path_split_head_tail OK")
