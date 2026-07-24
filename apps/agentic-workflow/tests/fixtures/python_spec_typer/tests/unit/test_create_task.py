from task_cli.application.create_task import create_task


def test_normalizes_title_inside_application_boundary() -> None:
    assert create_task("  ship release  ", "alice").title == "ship release"
