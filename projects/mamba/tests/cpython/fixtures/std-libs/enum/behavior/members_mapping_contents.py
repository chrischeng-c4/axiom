# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "members_mapping_contents"
# subject = "enum.Enum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Enum: __members__ maps every name (including aliases) to its member; canonical names are present and len matches the canonical member count"""
import enum


class Color(enum.Enum):
    RED = 1
    GREEN = 2
    BLUE = 3
    ROUGE = 1  # alias for RED


# __members__ includes canonical names and aliases.
assert "RED" in Color.__members__, "canonical name present"
assert "ROUGE" in Color.__members__, "alias name present in __members__"
assert Color.__members__["ROUGE"] is Color.RED, "alias maps to its canonical member"
# __members__ length counts every name (3 canonical + 1 alias).
assert len(Color.__members__) == 4, f"members len = {len(Color.__members__)!r}"
# Iteration excludes the alias.
assert len(list(Color)) == 3, f"canonical member count = {len(list(Color))!r}"

print("members_mapping_contents OK")
