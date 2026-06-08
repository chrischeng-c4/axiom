# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "dict_annotated_word_count"
# subject = "typing.Dict"
# kind = "semantic"
# xfail = "mamba diverges on the typing generic-alias runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Dict: a Dict[str,int]-annotated function returns a plain dict: _count_words('a b a') returns {'a':2,'b':1}"""
import typing
from typing import Dict


def _count_words(text: str) -> Dict[str, int]:
    result: Dict[str, int] = {}
    for w in text.split():
        result[w] = result.get(w, 0) + 1
    return result


assert _count_words("a b a") == {"a": 2, "b": 1}, f"word count = {_count_words('a b a')!r}"

print("dict_annotated_word_count OK")
