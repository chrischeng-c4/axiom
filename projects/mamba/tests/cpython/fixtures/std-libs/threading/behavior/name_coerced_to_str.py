# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "behavior"
# case = "name_coerced_to_str"
# subject = "threading.Thread"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: a non-string Thread name argument (123) is coerced to the str '123'"""
import threading

coerced = threading.Thread(name=123)
assert coerced.name == "123", f"coerced name = {coerced.name!r}"

print("name_coerced_to_str OK")
