# main.py
import time
import psycopg2
from psycopg2.extras import RealDictCursor
from worker import process_job

# Optional import for multi-threading (future use)
from concurrent.futures import ThreadPoolExecutor

import os
import psycopg2

POSTGRES_URL = os.environ.get("POSTGRES_URL")
if not POSTGRES_URL:
    raise RuntimeError("POSTGRES_URL environment variable not set")

def get_db_conn():
    return psycopg2.connect(POSTGRES_URL)

USE_MULTITHREAD = False  # 🔹 toggle this later
MAX_WORKERS = 2          # 🔹 adjust when enabling multi-thread

def get_next_job(conn):
    with conn.cursor(cursor_factory=RealDictCursor) as cur:
        cur.execute("""
            SELECT id, input FROM jobs
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        """)
        job = cur.fetchone()
        if job:
            cur.execute("UPDATE jobs SET status='running' WHERE id=%s", (job['id'],))
            conn.commit()
            return job
        return None

def single_thread_main():
    conn = psycopg2.connect(**DB_CONFIG)
    print("[Dispatcher] Running in single-thread mode")

    while True:
        job = get_next_job(conn)
        if job:
            print(f"[Dispatcher] Processing job {job['id']} sequentially...")
            process_job(conn, job)  # 🔹 Sequential execution
        else:
            time.sleep(2)

# This is capped by postgres connection pool size, GPU memory, and CPU
def multi_thread_main():
    print("[Dispatcher] Running in multi-thread mode")
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
        main_conn = psycopg2.connect(**DB_CONFIG)
        while True:
            job = get_next_job(main_conn)
            if job:
                print(f"[Dispatcher] Submitting job {job['id']} to worker...")
                executor.submit(threaded_job, job)
            else:
                time.sleep(2)

def threaded_job(job):
    # Each thread has its own connection
    conn = psycopg2.connect(**DB_CONFIG)
    process_job(conn, job)
    conn.close()

if __name__ == "__main__":
    if USE_MULTITHREAD:
        multi_thread_main()
    else:
        single_thread_main()
