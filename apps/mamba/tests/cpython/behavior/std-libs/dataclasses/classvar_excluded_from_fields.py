# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "behavior"
# case = "classvar_excluded_from_fields"
# subject = "dataclasses.dataclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: a typing.ClassVar-annotated attribute is excluded from dataclass field processing while plain-annotated fields are kept"""
import dataclasses
import typing


@dataclasses.dataclass
class WithClassVar:
    count: typing.ClassVar[int] = 0
    name: str = ""


cv_fields = [f.name for f in dataclasses.fields(WithClassVar)]
assert "count" not in cv_fields, "ClassVar excluded from fields()"
assert "name" in cv_fields, "name included"

print("classvar_excluded_from_fields OK")
