from typing import Any

__all__ = ["Task", "AsyncResult", "TaskSignature", "Chain", "Group", "GroupResult", "Chord", "init", "create_task"]

class Task:
    """A task that can be executed asynchronously

Created via the @task decorator. Provides methods to execute tasks
and create workflow signatures."""
    def __init__(self, name: str, queue: str = "default", max_retries: int = 3, retry_delay_secs: float = 1.0) -> None:
        ...
    def delay(self, args: tuple[Any, ...], kwargs: dict[Any, Any] | None, *args, **kwargs) -> Any:
        """Send task for async execution with positional args"""
        ...
    def apply_async(self, args: tuple[Any, ...], countdown: float | None = None, eta: str | None = None, kwargs: dict[Any, Any] | None, *args, **kwargs) -> Any:
        """Send task with options"""
        ...
    def s(self, args: tuple[Any, ...], kwargs: dict[Any, Any] | None, *args, **kwargs) -> Any:
        """Create a signature for this task (for workflows)"""
        ...
    @property
    def name(self) -> str:
        """Get task name"""
        ...
    @property
    def queue(self) -> str:
        """Get queue name"""
        ...

class AsyncResult:
    """Handle to track async task execution"""
    @property
    def task_id(self) -> str:
        """Get task ID as string"""
        ...
    def ready(self) -> Any:
        """Check if task is complete"""
        ...
    def get(self, timeout: float | None = None) -> Any:
        """Get result (waits for completion)"""
        ...
    def state(self) -> Any:
        """Get current state without waiting"""
        ...
    def info(self) -> Any:
        """Get full result object (includes state, result, error, timestamps)"""
        ...

class TaskSignature:
    """Python wrapper for TaskSignature"""
    @property
    def task_name(self) -> str:
        """Get task name"""
        ...
    def apply_async(self) -> Any:
        """Execute this signature"""
        ...

class Chain:
    """Python wrapper for Chain workflow"""
    def __init__(self, tasks: list[Any]) -> None:
        ...
    def apply_async(self) -> Any:
        """Execute the chain"""
        ...
    def __len__(self) -> int:
        """Get number of tasks in chain"""
        ...

class Group:
    """Python wrapper for Group workflow"""
    def __init__(self, tasks: list[Any]) -> None:
        ...
    def apply_async(self) -> Any:
        """Execute the group"""
        ...
    def __len__(self) -> int:
        """Get number of tasks in group"""
        ...

class GroupResult:
    """Result handle for a Group workflow"""
    @property
    def task_ids(self) -> list[str]:
        """Get list of task IDs"""
        ...
    def get(self, timeout: float | None = None) -> Any:
        """Get results for all tasks (waits for all to complete)"""
        ...

class Chord:
    """Python wrapper for Chord workflow"""
    def __init__(self, header: list[Any], callback: Any) -> None:
        ...
    def apply_async(self) -> Any:
        """Execute the chord"""
        ...

def init(redis_url: str, broker_type: str | None = None, nats_url: str | None = None, pubsub_project_id: str | None = None, pubsub_topic: str | None = None, pubsub_subscription: str | None = None) -> Any:
    ...

def create_task(name: str, queue: str = "default", max_retries: int = 3, retry_delay_secs: float = 1.0) -> Any:
    """Create a task (helper for Python decorator)"""
    ...

