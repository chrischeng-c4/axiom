from celery import Celery
app = Celery('bench', broker='redis://localhost:6379/0', backend='redis://localhost:6379/1')
app.conf.update(task_ignore_result=False, result_expires=300, worker_prefetch_multiplier=4)
@app.task
def echo(x):
    return x
