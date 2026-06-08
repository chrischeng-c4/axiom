"""
Example demonstrating BackgroundTasks usage in data-bridge-api.

This example shows how to use background tasks to perform operations
after the HTTP response is sent, similar to FastAPI's BackgroundTasks.
"""

from typing import Annotated
from cclab.quasar import App, Body, Depends, BackgroundTasks, get_background_tasks
from cclab.quasar import BaseModel
import asyncio


app = App(
    title="Background Tasks Demo",
    version="1.0.0",
    description="Demonstrates background task execution"
)


class EmailRequest(BaseModel):
    """Email request model."""
    to: str
    subject: str
    body: str


class SignupRequest(BaseModel):
    """User signup request model."""
    email: str
    username: str


# Simulated email sending function
async def send_email(to: str, subject: str, body: str):
    """Simulate sending an email (async operation)."""
    print(f"Sending email to {to}")
    await asyncio.sleep(0.5)  # Simulate email sending delay
    print(f"Email sent to {to}: {subject}")


# Simulated analytics logging
def log_analytics_event(event_type: str, user_id: int, metadata: dict):
    """Log analytics event (sync operation)."""
    print(f"Analytics: {event_type} for user {user_id}")
    print(f"Metadata: {metadata}")


@app.post("/send-email")
async def send_email_endpoint(
    email_req: Annotated[EmailRequest, Body()],
    background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
):
    """
    Send an email in the background.

    The response is returned immediately while the email is sent
    after the response is delivered to the client.
    """
    # Queue the email to be sent in the background
    background.add_task(
        send_email,
        to=email_req.to,
        subject=email_req.subject,
        body=email_req.body
    )

    # Return immediately without waiting for email
    return {
        "message": "Email queued for sending",
        "recipient": email_req.to
    }


@app.post("/signup")
async def signup_endpoint(
    signup: Annotated[SignupRequest, Body()],
    background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
):
    """
    User signup with multiple background tasks.

    After signup, we:
    1. Send welcome email (async)
    2. Log analytics event (sync)
    3. Update user stats (async)
    """
    # Simulate saving user to database
    user_id = 12345

    # Queue welcome email
    background.add_task(
        send_email,
        to=signup.email,
        subject="Welcome!",
        body=f"Welcome {signup.username}!"
    )

    # Queue analytics logging
    background.add_task(
        log_analytics_event,
        event_type="user_signup",
        user_id=user_id,
        metadata={
            "email": signup.email,
            "username": signup.username,
            "source": "web"
        }
    )

    # Queue another async task
    async def update_user_stats():
        await asyncio.sleep(0.1)
        print(f"Updated stats for user {user_id}")

    background.add_task(update_user_stats)

    # Return immediately
    return {
        "message": "Signup successful",
        "user_id": user_id,
        "email": signup.email
    }


@app.post("/process-order")
async def process_order(
    order_id: int,
    background: Annotated[BackgroundTasks, Depends(get_background_tasks)]
):
    """
    Process an order with chained background tasks.

    Tasks execute in order:
    1. Process payment
    2. Send confirmation email
    3. Update inventory
    """
    def process_payment(order_id: int):
        print(f"Processing payment for order {order_id}")

    async def send_confirmation(order_id: int):
        await asyncio.sleep(0.2)
        print(f"Sent confirmation for order {order_id}")

    def update_inventory(order_id: int):
        print(f"Updated inventory for order {order_id}")

    # Add tasks in execution order
    background.add_task(process_payment, order_id)
    background.add_task(send_confirmation, order_id)
    background.add_task(update_inventory, order_id)

    return {
        "message": "Order received and processing",
        "order_id": order_id
    }


if __name__ == "__main__":
    print("Background Tasks Example")
    print("=" * 50)
    print("\nThis example demonstrates:")
    print("1. Using BackgroundTasks with Depends()")
    print("2. Queueing async and sync tasks")
    print("3. Multiple background tasks per request")
    print("4. Chained operations that run in order")
    print("\nRoutes:")
    print("  POST /send-email - Send email in background")
    print("  POST /signup - User signup with multiple tasks")
    print("  POST /process-order - Process order with chained tasks")
