# worker.py
import json
import ollama
from pydantic import BaseModel

def fetch_job(conn, job_id):
    with conn.cursor() as cur:
        cur.execute("SELECT input FROM jobs WHERE id=%s", (job_id,))
        return cur.fetchone()

def update_job_output(conn, job_id, output):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET output=%s
            WHERE id=%s
        """, (status, json.dumps(output), json.dumps(error), job_id))
        conn.commit()

def finish_processing(conn, job_id, output):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET status='completing', output=%s
            WHERE id=%s
        """, (json.dumps(output), job_id))
        conn.commit()

def fail_job(conn, job_id, error):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET status='failed', error=%s
            WHERE id=%s
        """, (json.dumps(error), job_id))

def complete_job(conn, job_id):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET status='completed'
            WHERE id=%s
        """, (job_id))

def parse_image_id(image_path):
    """
    Extracts the id from an image path of the form "/xxx/xxxx/{id}_{name}.{data_type}"
    Example: "/data/input/abc123_foo.png" -> "abc123"
    """
    import os
    filename = os.path.basename(image_path)
    if "_" in filename:
        id_part = filename.split("_", 1)[0]
        return id_part
    return None

class TaskTiming(BaseModel):
    start_image: str
    end_image: str

def run_inference(prompt, image_paths):
    start_image = None
    end_image = None

    res = ollama.chat(
        model="llava:7b",
        messages=[
            {"role": "user", "content": prompt, "images": image_paths}
        ]
        format=TaskTiming.model_json_schema()
    )
    
    output = TaskTiming.model_validate_json(res.message.content)
    start_image = parse_image_id(output.start_image) if output.start_image else None
    end_image = parse_image_id(output.end_image) if output.end_image else None

    return {
        "start_image_id": start_image,
        "end_image_id": end_image
    }

def send_webhook(webhook_url, job_id, data, error):
    if webhook_url is None:
        return

    import requests

    payload = {
        "job_id": job_id,
        "data": data,
        "error": error
    }
    requests.post(webhook_url, json=payload, timeout=10)

def process_job(conn, job: dict):
    print(f"[Worker] Processing job {job['id']}")
    inputs = job['input']
    images_paths = "/data/input/"+ job['id']+"/"

    try:
        results = run_inference(inputs['prompt'], images_paths)
    except Exception as e:
        print(f"[Worker] Error processing job {job['id']}: {e}")
        err = {
            "code": 100,
            "message": str(e)
        }
        fail_job(conn, job['id'], err)
        send_webhook(inputs['webhook_url'], job['id'], None, err)
        return

    finish_processing(conn, job['id'], results)

    try:
        send_webhook(inputs['webhook_url'], job['id'], results, None)
        print(f"[Worker] Webhook sent successfully")

    except Exception as e:
        print(f"[Worker] Failed to send webhook: {e}")
        err = {
            "code": 200,
            "message": str(e)
        }
        fail_job(conn, job['id'], err)

    print(f"[Worker] Job {job['id']} completed.")
