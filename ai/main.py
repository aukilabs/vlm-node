# main.py
import time
import psycopg2
from psycopg2.extras import RealDictCursor
from worker import process_job

# Optional import for multi-threading (future use)
from concurrent.futures import ThreadPoolExecutor

DB_CONFIG = {
    "dbname": "mydb",
    "user": "postgres",
    "password": "password",
    "host": "localhost",
    "port": 5432
}

USE_MULTITHREAD = False  # 🔹 toggle this later
MAX_WORKERS = 2          # 🔹 adjust when enabling multi-thread

def get_next_job(conn):
    with conn.cursor(cursor_factory=RealDictCursor) as cur:
        cur.execute("""
            SELECT * FROM jobs
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        """)
        job = cur.fetchone()
        if job:
            cur.execute("UPDATE jobs SET status='started' WHERE id=%s", (job['id'],))
            conn.commit()
        return job

def single_thread_main():
    conn = psycopg2.connect(**DB_CONFIG)
    print("[Dispatcher] Running in single-thread mode")

    while True:
        job = get_next_job(conn)
        if job:
            print(f"[Dispatcher] Processing job {job['id']} sequentially...")
            process_job(conn, job['id'])  # 🔹 Sequential execution
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
                executor.submit(threaded_job, job['id'])
            else:
                time.sleep(2)

def threaded_job(job_id):
    # Each thread has its own connection
    conn = psycopg2.connect(**DB_CONFIG)
    process_job(conn, job_id)
    conn.close()

if __name__ == "__main__":
    if USE_MULTITHREAD:
        multi_thread_main()
    else:
        single_thread_main()
