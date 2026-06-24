import asyncio, time
from datetime import timedelta
from temporalio import workflow, activity
from temporalio.client import Client
from temporalio.worker import Worker, UnsandboxedWorkflowRunner

@activity.defn
async def echo(x: str) -> str:
    return x

@workflow.defn
class EchoWF:
    @workflow.run
    async def run(self, x: str) -> str:
        return await workflow.execute_activity(echo, x, start_to_close_timeout=timedelta(seconds=10))

async def main():
    client = await Client.connect("127.0.0.1:7233")
    N = 500
    async with Worker(client, task_queue="bench", workflows=[EchoWF], activities=[echo],
                      max_concurrent_activities=8, max_concurrent_workflow_tasks=8,
                      workflow_runner=UnsandboxedWorkflowRunner()):
        t0 = time.time()
        handles = await asyncio.gather(*[
            client.start_workflow(EchoWF.run, f"v{i}", id=f"wf-{i}", task_queue="bench")
            for i in range(N)])
        t1 = time.time()
        await asyncio.gather(*[h.result() for h in handles])
        t2 = time.time()
        print(f"temporal: N={N} (dev server, sqlite-backed) — durable workflow engine")
        print(f"  submit:    {t1-t0:.2f}s ({N/(t1-t0):.0f} starts/s)")
        print(f"  end-to-end:{t2-t0:.2f}s ({N/(t2-t0):.0f} workflows/s completed)")

asyncio.run(main())
