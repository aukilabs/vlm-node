# main.py
import time
import psycopg
from worker import process_job, fail_job

# Optional import for multi-threading (future use)
from concurrent.futures import ThreadPoolExecutor

import os

POSTGRES_URL = os.environ.get("POSTGRES_URL")
if not POSTGRES_URL:
    raise RuntimeError("POSTGRES_URL environment variable not set")

def get_db_conn():
    return psycopg.connect(POSTGRES_URL)

USE_MULTITHREAD = False  # 🔹 toggle this later
MAX_WORKERS = 2          # 🔹 adjust when enabling multi-thread

def get_next_job(conn):
    with conn.cursor() as cur:
        cur.execute("""
            SELECT id, input FROM jobs
            WHERE job_status = 'pending'
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        """)
        job = cur.fetchone()
        if job:
            job = {
                "id": job[0],
                "input": job[1]
            }
            cur.execute("UPDATE jobs SET job_status='running', updated_at=now() WHERE id=%s", (job['id'],))
            conn.commit()
            return job
        return None

def single_thread_main():
    conn = get_db_conn()
    print("[Dispatcher] Running in single-thread mode")

    while True:
        job = get_next_job(conn)
        if job:
            print(f"[Dispatcher] Processing job {job['id']} sequentially...")
            try:
                process_job(conn, job)  # 🔹 Sequential execution
            except Exception as e:
                print(f"[Dispatcher] Error processing job {job['id']}: {e}")
                err = {
                    "code": 100,
                    "message": str(e)
                }
                fail_job(conn, job['id'], err)
            except KeyboardInterrupt:
                print("[Dispatcher] Keyboard interrupt received, cancelling job...")
                cancel_job(conn, job['id'])
                exit(0)
        else:
            time.sleep(2)

# This is capped by postgres connection pool size, GPU memory, and CPU
def multi_thread_main():
    print("[Dispatcher] Running in multi-thread mode")
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
        main_conn = get_db_conn()
        while True:
            job = get_next_job(main_conn)
            if job:
                print(f"[Dispatcher] Submitting job {job['id']} to worker...")
                executor.submit(threaded_job, job)
            else:
                time.sleep(2)

def threaded_job(job):
    # Each thread has its own connection
    conn = get_db_conn()
    process_job(conn, job)
    conn.close()

if __name__ == "__main__":
    if USE_MULTITHREAD:
        multi_thread_main()
    else:
        single_thread_main()
