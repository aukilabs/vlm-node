# worker.py
import os
from jobs import finish_processing, fail_job, complete_job
import vlm
from logger_config import get_logger

DATA_DIR = os.environ.get("DATA_DIR", "data")
INPUT_DIR = DATA_DIR + "/input/"
OUTPUT_DIR = DATA_DIR + "/output/"

# Setup logger
logger = get_logger("worker")

def process_job(conn, job: dict):
    logger.info("Processing job", extra={"job_id": job['id']})
    input_dir = INPUT_DIR + job['id'] + "/"
    output_dir = OUTPUT_DIR + job['id'] + "/"

    # Logic to process job
    vlm.run(conn, job, input_dir, output_dir)

    # When the job fails, call fail_job(conn, job['id'], err)

    # Upload output to domain server
    # Save output to output directory
    # Call upload_job_result(conn, job_id)
    # or if you don't want to upload
    # Call complete_job(conn, job_id)

    logger.info("Job completed", extra={"job_id": job['id']})
