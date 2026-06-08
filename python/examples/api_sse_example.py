"""
Example of Server-Sent Events (SSE) streaming with data-bridge API.

This example demonstrates:
1. Simple event streaming
2. Event types and IDs
3. Time-based event generation
4. Multiple SSE endpoints
"""

import asyncio
import time
from datetime import datetime
from cclab.quasar import App, ServerSentEvent, EventSourceResponse


app = App(title="SSE Example", version="1.0.0")


@app.get("/events/simple")
async def simple_events():
    """Simple event stream - sends 10 events."""
    async def generate():
        for i in range(10):
            await asyncio.sleep(0.5)  # Wait 0.5 seconds between events
            yield ServerSentEvent(data=f"Event {i}")

    return EventSourceResponse(generate())


@app.get("/events/counter")
async def counter_events():
    """Counter with event types and IDs."""
    async def generate():
        for i in range(20):
            await asyncio.sleep(1)  # Wait 1 second between events
            yield ServerSentEvent(
                data=f"Count: {i}",
                event="counter",
                id=str(i)
            )

    return EventSourceResponse(generate())


@app.get("/events/clock")
async def clock_events():
    """Real-time clock stream."""
    async def generate():
        for _ in range(60):  # Stream for 60 seconds
            current_time = datetime.now().strftime("%H:%M:%S")
            yield ServerSentEvent(
                data=current_time,
                event="time",
                id=str(int(time.time()))
            )
            await asyncio.sleep(1)

    return EventSourceResponse(generate())


@app.get("/events/status")
async def status_events():
    """Status updates with retry interval."""
    async def generate():
        statuses = ["starting", "processing", "complete"]
        for i, status in enumerate(statuses):
            await asyncio.sleep(2)
            yield ServerSentEvent(
                data=f"Status: {status}",
                event="status",
                id=str(i),
                retry=5000  # Client should retry after 5 seconds if disconnected
            )

    return EventSourceResponse(generate())


@app.get("/events/multiline")
async def multiline_events():
    """Events with multiline data."""
    async def generate():
        messages = [
            "Hello\nWorld",
            "This is\na multiline\nmessage",
            "Line 1\nLine 2\nLine 3\nLine 4"
        ]
        for i, message in enumerate(messages):
            await asyncio.sleep(1)
            yield ServerSentEvent(
                data=message,
                event="message",
                id=str(i)
            )

    return EventSourceResponse(generate())


@app.get("/")
async def root():
    """Serve HTML client for testing SSE endpoints."""
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>SSE Example Client</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 800px; margin: 20px auto; padding: 20px; }
            .endpoint { margin: 20px 0; padding: 10px; border: 1px solid #ddd; border-radius: 5px; }
            .events { background: #f5f5f5; padding: 10px; min-height: 100px; max-height: 300px; overflow-y: auto; font-family: monospace; }
            button { margin: 5px; padding: 5px 10px; }
            .event-line { margin: 2px 0; }
        </style>
    </head>
    <body>
        <h1>Server-Sent Events Example</h1>

        <div class="endpoint">
            <h2>Simple Events</h2>
            <button onclick="connectSimple()">Connect</button>
            <button onclick="disconnectSimple()">Disconnect</button>
            <div id="simple" class="events"></div>
        </div>

        <div class="endpoint">
            <h2>Counter with Types</h2>
            <button onclick="connectCounter()">Connect</button>
            <button onclick="disconnectCounter()">Disconnect</button>
            <div id="counter" class="events"></div>
        </div>

        <div class="endpoint">
            <h2>Real-time Clock</h2>
            <button onclick="connectClock()">Connect</button>
            <button onclick="disconnectClock()">Disconnect</button>
            <div id="clock" class="events"></div>
        </div>

        <div class="endpoint">
            <h2>Status Updates</h2>
            <button onclick="connectStatus()">Connect</button>
            <button onclick="disconnectStatus()">Disconnect</button>
            <div id="status" class="events"></div>
        </div>

        <script>
            let simpleSource, counterSource, clockSource, statusSource;

            function addEvent(elementId, message) {
                const div = document.getElementById(elementId);
                const line = document.createElement('div');
                line.className = 'event-line';
                line.textContent = new Date().toLocaleTimeString() + ': ' + message;
                div.appendChild(line);
                div.scrollTop = div.scrollHeight;
            }

            function connectSimple() {
                disconnectSimple();
                simpleSource = new EventSource('/events/simple');
                simpleSource.onmessage = (e) => addEvent('simple', e.data);
                simpleSource.onerror = () => addEvent('simple', 'Connection closed');
            }

            function disconnectSimple() {
                if (simpleSource) simpleSource.close();
            }

            function connectCounter() {
                disconnectCounter();
                counterSource = new EventSource('/events/counter');
                counterSource.addEventListener('counter', (e) => {
                    addEvent('counter', `ID ${e.lastEventId}: ${e.data}`);
                });
                counterSource.onerror = () => addEvent('counter', 'Connection closed');
            }

            function disconnectCounter() {
                if (counterSource) counterSource.close();
            }

            function connectClock() {
                disconnectClock();
                clockSource = new EventSource('/events/clock');
                clockSource.addEventListener('time', (e) => {
                    addEvent('clock', e.data);
                });
                clockSource.onerror = () => addEvent('clock', 'Connection closed');
            }

            function disconnectClock() {
                if (clockSource) clockSource.close();
            }

            function connectStatus() {
                disconnectStatus();
                statusSource = new EventSource('/events/status');
                statusSource.addEventListener('status', (e) => {
                    addEvent('status', `${e.data} (retry: 5000ms)`);
                });
                statusSource.onerror = () => addEvent('status', 'Connection closed');
            }

            function disconnectStatus() {
                if (statusSource) statusSource.close();
            }
        </script>
    </body>
    </html>
    """
    from cclab.quasar import HTMLResponse
    return HTMLResponse(html)


if __name__ == "__main__":
    print("Starting SSE example server...")
    print("Open http://localhost:8000 in your browser")
    print()
    print("Available endpoints:")
    print("  GET /events/simple    - Simple event stream")
    print("  GET /events/counter   - Counter with event types and IDs")
    print("  GET /events/clock     - Real-time clock")
    print("  GET /events/status    - Status updates with retry")
    print("  GET /events/multiline - Multiline message events")
    print()
    print("Or test with curl:")
    print("  curl http://localhost:8000/events/simple")
    print()

    app.run(host="0.0.0.0", port=8000)
