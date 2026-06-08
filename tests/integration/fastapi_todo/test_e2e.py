"""E2E tests for FastAPI Todo app using Playwright."""
import subprocess
import time
import sys
import signal
import requests
from playwright.sync_api import sync_playwright


BASE_URL = "http://127.0.0.1:8899"
APP_DIR = __file__.rsplit("/", 1)[0]


def wait_for_server(url, timeout=10):
    """Wait until the server responds."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            r = requests.get(url, timeout=1)
            if r.status_code == 200:
                return True
        except Exception:
            pass
        time.sleep(0.3)
    return False


def run_tests():
    # Start the server
    print("Starting uvicorn server...")
    server = subprocess.Popen(
        [sys.executable, "-m", "uvicorn", "main:app",
         "--host", "127.0.0.1", "--port", "8899"],
        cwd=APP_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    try:
        if not wait_for_server(BASE_URL):
            print("FAIL: Server did not start")
            server.kill()
            out, err = server.communicate()
            print("stdout:", out.decode())
            print("stderr:", err.decode())
            return False

        print("Server is up. Running E2E tests...\n")

        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            page = browser.new_page()

            # --- Test 1: Load page ---
            print("Test 1: Load page")
            page.goto(BASE_URL)
            assert page.title() == "Mamba Todo"
            assert page.locator("h1").text_content() == "Mamba Todo"
            print("  PASS\n")

            # --- Test 2: Empty state ---
            print("Test 2: Empty state shows message")
            empty = page.locator(".empty")
            assert empty.is_visible()
            assert "No todos yet" in empty.text_content()
            print("  PASS\n")

            # --- Test 3: Add a todo ---
            print("Test 3: Add a todo")
            page.fill("#todoInput", "Buy groceries")
            page.click('.add-form button[type="submit"]')
            page.wait_for_selector(".todo-item")
            items = page.locator(".todo-item")
            assert items.count() == 1
            assert "Buy groceries" in items.first.text_content()
            print("  PASS\n")

            # --- Test 4: Add a second todo ---
            print("Test 4: Add a second todo")
            page.fill("#todoInput", "Clean house")
            page.click('.add-form button[type="submit"]')
            page.wait_for_function("document.querySelectorAll('.todo-item').length === 2")
            items = page.locator(".todo-item")
            assert items.count() == 2
            print("  PASS\n")

            # --- Test 5: Toggle completion ---
            print("Test 5: Toggle completion")
            # The most recent todo is first (desc order)
            checkbox = items.first.locator('input[type="checkbox"]')
            checkbox.check()
            page.wait_for_selector(".todo-item.completed")
            completed = page.locator(".todo-item.completed")
            assert completed.count() >= 1
            print("  PASS\n")

            # --- Test 6: Delete a todo ---
            print("Test 6: Delete a todo")
            items = page.locator(".todo-item")
            count_before = items.count()
            items.first.locator(".delete-btn").click()
            page.wait_for_function(
                f"document.querySelectorAll('.todo-item').length === {count_before - 1}"
            )
            items = page.locator(".todo-item")
            assert items.count() == count_before - 1
            print("  PASS\n")

            # --- Test 7: API direct check ---
            print("Test 7: API returns JSON")
            response = page.request.get(f"{BASE_URL}/api/todos")
            assert response.status == 200
            data = response.json()
            assert isinstance(data, list)
            assert len(data) == 1  # one remaining after delete
            assert data[0]["title"] == "Buy groceries"
            print("  PASS\n")

            browser.close()

        print("All E2E tests passed!")
        return True

    finally:
        server.send_signal(signal.SIGTERM)
        server.wait(timeout=5)


if __name__ == "__main__":
    # Clean up old data before test
    import asyncio
    async def cleanup():
        from database import engine
        from sqlalchemy import text
        async with engine.begin() as conn:
            await conn.execute(text("DROP TABLE IF EXISTS todos"))
        await engine.dispose()

    sys.path.insert(0, APP_DIR)
    asyncio.run(cleanup())

    ok = run_tests()
    sys.exit(0 if ok else 1)
