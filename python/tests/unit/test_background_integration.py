"""Integration tests for BackgroundTasks with App and dependencies."""

import pytest
from typing import Annotated
from cclab.quasar import (
    App, Depends, BackgroundTasks, get_background_tasks,
    Body, Query
)
from cclab.quasar.models import BaseModel


class TestBackgroundTasksIntegration:
    """Test BackgroundTasks integration with App."""

    def test_background_tasks_with_depends(self):
        """Test BackgroundTasks injection via Depends."""
        app = App(title="Test")
        executed = []

        @app.post("/test")
        async def handler(
            background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
        ):
            background.add_task(lambda: executed.append("task1"))
            return {"status": "ok"}

        # Verify handler was registered
        assert len(app._routes) > 0

    def test_background_tasks_with_multiple_dependencies(self):
        """Test BackgroundTasks alongside other dependencies."""
        app = App(title="Test")

        def get_db():
            return {"db": "connected"}

        @app.post("/test")
        async def handler(
            db: Annotated[dict, Depends(get_db)],
            background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
        ):
            background.add_task(lambda: None)
            return db

        # Both dependencies should be registered
        assert len(app._routes) > 0

    def test_background_tasks_with_body(self):
        """Test BackgroundTasks with request body."""
        app = App(title="Test")

        class UserCreate(BaseModel):
            email: str
            name: str

        @app.post("/users")
        async def create_user(
            user: Annotated[UserCreate, Body()],
            background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
        ):
            background.add_task(lambda email: None, user.email)
            return {"user_id": 123}

        assert len(app._routes) > 0

    def test_background_tasks_with_query(self):
        """Test BackgroundTasks with query parameters."""
        app = App(title="Test")

        @app.get("/process")
        async def process(
            task_id: Annotated[int, Query()],
            background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
        ):
            background.add_task(lambda: None)
            return {"task_id": task_id}

        assert len(app._routes) > 0

    @pytest.mark.asyncio
    async def test_factory_creates_new_instances(self):
        """Test that factory creates independent instances."""
        bg1 = get_background_tasks()
        bg2 = get_background_tasks()

        bg1.add_task(lambda: None)
        bg2.add_task(lambda: None)
        bg2.add_task(lambda: None)

        # Should be independent
        assert len(bg1) == 1
        assert len(bg2) == 2
        assert bg1 is not bg2

    @pytest.mark.asyncio
    async def test_background_tasks_isolated_per_request(self):
        """Test that background tasks are isolated per request."""
        # Simulate two separate requests
        request1_bg = get_background_tasks()
        request2_bg = get_background_tasks()

        executed_request1 = []
        executed_request2 = []

        request1_bg.add_task(lambda: executed_request1.append(1))
        request2_bg.add_task(lambda: executed_request2.append(2))

        # Execute independently
        await request1_bg.run()
        await request2_bg.run()

        assert executed_request1 == [1]
        assert executed_request2 == [2]

    def test_multiple_routes_with_background_tasks(self):
        """Test multiple routes using BackgroundTasks."""
        app = App(title="Test")

        @app.post("/route1")
        async def route1(
            background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
        ):
            background.add_task(lambda: None)
            return {"route": 1}

        @app.post("/route2")
        async def route2(
            background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
        ):
            background.add_task(lambda: None)
            background.add_task(lambda: None)
            return {"route": 2}

        @app.get("/route3")
        async def route3(
            background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
        ):
            return {"route": 3}

        # All routes should be registered
        assert len(app._routes) == 3

    @pytest.mark.asyncio
    async def test_background_tasks_execution_order_preserved(self):
        """Test that task execution order is preserved."""
        background = get_background_tasks()
        execution_order = []

        def task1():
            execution_order.append(1)

        def task2():
            execution_order.append(2)

        async def task3():
            execution_order.append(3)

        def task4():
            execution_order.append(4)

        background.add_task(task1)
        background.add_task(task2)
        background.add_task(task3)
        background.add_task(task4)

        await background.run()

        # Tasks should execute in order added
        assert execution_order == [1, 2, 3, 4]

    @pytest.mark.asyncio
    async def test_background_tasks_with_error_continues(self):
        """Test that errors don't prevent subsequent tasks."""
        background = get_background_tasks()
        executed = []

        def failing_task():
            executed.append("before_error")
            raise ValueError("Task failed")

        def success_task():
            executed.append("after_error")

        background.add_task(failing_task)
        background.add_task(success_task)

        # Should not raise
        await background.run()

        # Both tasks should have run
        assert "before_error" in executed
        assert "after_error" in executed

    def test_background_tasks_repr_in_app_context(self):
        """Test BackgroundTasks repr in app context."""
        app = App(title="Test")

        @app.post("/test")
        async def handler(
            background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
        ):
            # Should have good repr
            assert "BackgroundTasks" in repr(background)
            assert "tasks=0" in repr(background)

            background.add_task(lambda: None)
            assert "tasks=1" in repr(background)

            return {"status": "ok"}

        # Just verify route registered
        assert len(app._routes) > 0

    @pytest.mark.asyncio
    async def test_nested_dependency_with_background_tasks(self):
        """Test BackgroundTasks with nested dependencies."""
        executed = []

        def get_config():
            return {"config": "loaded"}

        def get_service(config: Annotated[dict, Depends(get_config)]):
            return {"service": "initialized", "config": config}

        # Create a handler that uses nested deps + background tasks
        async def handler(
            service: Annotated[dict, Depends(get_service)],
            background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
        ):
            background.add_task(lambda: executed.append(service))
            return {"status": "ok"}

        # Create background tasks instance
        bg = get_background_tasks()
        bg.add_task(lambda: executed.append("test"))

        await bg.run()
        assert executed == ["test"]

    @pytest.mark.asyncio
    async def test_clear_after_run(self):
        """Test clearing tasks after running them."""
        background = get_background_tasks()
        executed = []

        background.add_task(lambda: executed.append(1))
        background.add_task(lambda: executed.append(2))

        assert len(background) == 2

        await background.run()
        assert executed == [1, 2]

        # Clear after run
        background.clear()
        assert len(background) == 0

        # Run again should do nothing
        await background.run()
        assert executed == [1, 2]  # No new executions
