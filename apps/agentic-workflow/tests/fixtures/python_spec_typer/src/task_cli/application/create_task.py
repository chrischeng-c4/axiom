from task_cli.domain.task import Task


def create_task(title: str, actor: str) -> Task:
    normalized = title.strip()
    if not normalized:
        raise ValueError("title must not be empty")
    if "/" in normalized or "\\" in normalized or ".." in normalized:
        raise ValueError("title contains a forbidden path token")
    return Task(title=normalized, actor=actor)
