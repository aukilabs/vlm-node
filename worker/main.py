# main.py
import time
import psycopg
from jobs import get_next_job, fail_job, cancel_job
from worker import process_job
from logger_config import get_logger

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

# Setup logger
logger = get_logger("dispatcher")

def single_thread_main():
    conn = get_db_conn()
    logger.info("Running in single-thread mode")

    while True:
        job = get_next_job(conn)
        if job:
            logger.info("Processing job sequentially", extra={"job_id": job['id']})
            try:
                process_job(conn, job)  # 🔹 Sequential execution
            except Exception as e:
                logger.error("Error processing job", extra={"job_id": job['id'], "error": str(e)})
                err = {
                    "code": 100,
                    "message": str(e)
                }
                fail_job(conn, job['id'], err)
            except KeyboardInterrupt:
                logger.info("Keyboard interrupt received, cancelling job", extra={"job_id": job['id']})
                cancel_job(conn, job['id'])
                exit(0)
        else:
            time.sleep(2)

# This is capped by postgres connection pool size, GPU memory, and CPU
def multi_thread_main():
    logger.info("Running in multi-thread mode")
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as executor:
        main_conn = get_db_conn()
        while True:
            job = get_next_job(main_conn)
            if job:
                logger.info("Submitting job to worker", extra={"job_id": job['id']})
                executor.submit(threaded_job, job)
            else:
                time.sleep(2)

def threaded_job(job):
    # Each thread has its own connection
    conn = get_db_conn()
    process_job(conn, job)
    conn.close()

import threading
from http.server import BaseHTTPRequestHandler, HTTPServer

class HealthHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/health":
            self.send_response(200)
            self.send_header("Content-type", "text/plain")
            self.end_headers()
            self.wfile.write(b"ok")
        else:
            self.send_response(404)
            self.end_headers()

def run_health_server():
    server = HTTPServer(("0.0.0.0", 8081), HealthHandler)
    logger.info("Health server running on port 8081")
    server.serve_forever()

if __name__ == "__main__":
    # Start health server in a background thread
    health_thread = threading.Thread(target=run_health_server, daemon=True)
    health_thread.start()

    if USE_MULTITHREAD:
        multi_thread_main()
    else:
        single_thread_main()
