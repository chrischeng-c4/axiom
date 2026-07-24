from dataclasses import dataclass


@dataclass(frozen=True)
class Task:
    title: str
    actor: str
