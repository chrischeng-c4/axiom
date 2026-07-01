# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "method_resolution"
# dimension = "type"
# case = "abstract_method_called_directly"
# subject = "abstract method instantiation contract"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: instantiating a class with an
abstractmethod must raise TypeError.

CPython 3.12: raises TypeError "Can't instantiate abstract class".
Mamba: should also raise but may silently construct (see
project_mamba_class_machinery_silent_divergences for related class
silent-divergence patterns).
"""

from abc import ABC, abstractmethod


class Base(ABC):
    @abstractmethod
    def required(self) -> int: ...


try:
    inst = Base()  # type: ignore[abstract]
    print("no_typeerror:", type(inst).__name__)
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
