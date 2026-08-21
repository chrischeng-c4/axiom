# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "behavior"
# case = "while_walrus_holds_last_value_after_loop"
# subject = ":="
# kind = "semantic"
# xfail = "mamba walrus-in-while target reads 0 after loop exit; matches the legacy behavior.py mamba-xfail (val2 after loop = 0)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: a walrus in a while-condition reads from the enclosing scope and, after the loop exits on a falsy value, the target still holds that last assigned (falsy) value"""
# := in a while-condition binds each element; after the loop exits on the
# falsy 0, the target still holds that last assigned (falsy) value.
data2 = [10, 20, 0, 30]  # 0 is falsy -> loop stops
idx2 = 0
acc2 = 0
while idx2 < len(data2) and (val2 := data2[idx2]):
    acc2 += val2
    idx2 += 1
assert acc2 == 30, f"acc = {acc2!r}"  # 10 + 20, stops at 0
assert val2 == 0, f"val2 after loop = {val2!r}"  # last assigned value

print("while_walrus_holds_last_value_after_loop OK")
