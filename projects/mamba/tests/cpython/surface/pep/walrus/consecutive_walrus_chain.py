# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "walrus"
# dimension = "surface"
# case = "consecutive_walrus_chain"
# subject = ":="
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
""":=: consecutive walrus statements each build on the previous binding: (a:=1), (b:=a+1), (c:=b+1) leaves c == 3"""
# Consecutive := statements each build on the previous binding.
_ = (a := 1)
_ = (b := a + 1)
_ = (c := b + 1)
assert c == 3, f"chain = {c!r}"

print("consecutive_walrus_chain OK")
