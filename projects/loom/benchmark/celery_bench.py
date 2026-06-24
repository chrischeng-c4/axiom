import time
from celery_app import echo
N=500
t0=time.time()
results=[echo.delay(i) for i in range(N)]   # async submit
t1=time.time()
done=[r.get(timeout=60) for r in results]    # wait all
t2=time.time()
assert len(done)==N and done[0]==0
print(f"celery: N={N} workers=4 (redis broker)")
print(f"  submit:    {t1-t0:.2f}s ({N/(t1-t0):.0f} submits/s)")
print(f"  end-to-end:{t2-t0:.2f}s ({N/(t2-t0):.0f} tasks/s completed)")
