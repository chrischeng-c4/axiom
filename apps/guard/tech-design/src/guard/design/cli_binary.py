"""Executable design for the Guard binary entrypoint."""

__aw_artifact_id__ = "artifact:guard/design-cli-binary"


def binary_execution_order() -> tuple[str, ...]:
    return ("parse", "dispatch", "print_report", "exit_from_report")


def binary_exit_range() -> tuple[int, int]:
    return (0, 255)
