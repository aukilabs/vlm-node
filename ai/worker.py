# worker.py
import json
import ollama
import os
import re
import sys
import requests
from jobs import finish_processing, fail_job, complete_job

DATA_DIR = os.environ.get("DATA_DIR", "data")
INPUT_DIR = DATA_DIR + "/input/"
OUTPUT_DIR = DATA_DIR + "/output/"

def process_job(conn, job: dict):
    print(f"[Worker] Processing job {job['id']}")
    inputs = job['input']
    input_dir = INPUT_DIR + job['id'] + "/"
    output_dir = OUTPUT_DIR + job['id'] + "/"

    # Logic to process job

    # Upload output to domain server
    # Save output to output directory
    # Call upload_job_result(conn, job_id)

    print(f"[Worker] Job {job['id']} completed.")
