"""Stability contract for Guard's folded dynamic evidence projection."""

from guard_contract import assert_dynamic_evidence, run_dynamic_adapters

DIMENSION = "stability"


def _normalized_arguments(tool: str, command: list[object]) -> tuple[str, ...]:
    arguments = [str(value) for value in command[1:]]
    path_flag = {"rig": "--scenario", "meter": "--target"}.get(tool)
    if path_flag is not None:
        index = arguments.index(path_flag)
        arguments[index + 1] = "<fixture-path>"
    return tuple(arguments)


def _projection(report: dict[str, object]) -> tuple[tuple[object, ...], ...]:
    evidence = report["evidence"]
    assert isinstance(evidence, list)
    projected = []
    for item in evidence:
        assert isinstance(item, dict)
        tool = str(item["tool"])
        command = item["command"]
        folded_report = item["report"]
        assert isinstance(command, list)
        assert isinstance(folded_report, dict)
        summary = folded_report.get("summary")
        assert isinstance(summary, dict)
        projected.append(
            (
                tool,
                _normalized_arguments(tool, command),
                item.get("status"),
                item.get("clean"),
                item.get("exit_code"),
                item.get("finding_count"),
                folded_report.get("schema_version"),
                folded_report.get("clean"),
                summary.get("total"),
            )
        )
    return tuple(sorted(projected))


def verify() -> list[str]:
    tools = ("vat", "rig", "meter")
    first, first_traces, first_expectations = run_dynamic_adapters(tools)
    second, second_traces, second_expectations = run_dynamic_adapters(tools)
    assertions = assert_dynamic_evidence(
        first,
        first_traces,
        expectations=first_expectations,
    )
    assertions.extend(
        assert_dynamic_evidence(
            second,
            second_traces,
            expectations=second_expectations,
        )
    )
    if _projection(first) != _projection(second):
        raise AssertionError("equivalent adapter runs changed folded evidence fields")
    assertions.append(
        "adapter command grammar and folded evidence remain stable across fresh runs"
    )
    return assertions
