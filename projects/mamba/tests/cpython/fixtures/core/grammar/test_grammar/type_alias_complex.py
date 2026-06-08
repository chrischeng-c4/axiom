# RUN: parse

# PEP 695: type aliases
type Vector = list[float]
type Matrix[T] = list[list[T]]
type Alias[T: int] = list[T]
type Callback[**P] = Callable[P, None]
