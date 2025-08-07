# worker.py
import json
import ollama
from pydantic import BaseModel
from pathlib import Path
import os
import re

def finish_processing(conn, job_id, output):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='completing', output=%s
            WHERE id=%s
        """, (json.dumps(output), job_id))
        conn.commit()

def fail_job(conn, job_id, error):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='failed', error=%s
            WHERE id=%s
        """, (json.dumps(error), job_id))
        conn.commit()

def complete_job(conn, job_id):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='completed'
            WHERE id=%s
        """, (job_id,))
        conn.commit()

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

    print(f"[Worker] Running inference for {len(image_paths)} images: {image_paths}")

    res = ollama.chat(
        model="llava:7b",
        messages=[
            {"role": "user", "content": prompt, "images": image_paths}
        ],
        format=TaskTiming.model_json_schema()
    )
    
    output = TaskTiming.model_validate_json(res.message.content)

    print(f"[Worker] Inference output: {output}")
    start_image = parse_image_id(output.start_image) if output.start_image else None
    end_image = parse_image_id(output.end_image) if output.end_image else None

    print(f"[Worker] Inference completed")

    return {
        "start_image_id": start_image,
        "end_image_id": end_image
    }

def send_webhook(webhook_url, job_id, data, error):
    if webhook_url is None or webhook_url == "":
        print(f"[Worker] Webhook URL is empty")
        return

    import requests

    payload = {
        "job_id": job_id,
        "data": data,
        "error": error
    }
    try:
        response = requests.post(webhook_url, json=payload, timeout=10)
        if response.status_code != 200:
            print(f"[Worker] Failed to send webhook: {response.status_code}")
            print(f"[Worker] Response: {response.text}")
            raise Exception(f"Failed to send webhook: {response.text}")
        else:
            print(f"[Worker] Webhook sent successfully")
    except Exception as e:
        raise e

def find_images(input_dir):
    image_paths = []
    valid_exts = ('.jpg', '.jpeg', '.png')

    for filename in os.listdir(input_dir):
        lower_name = filename.lower()

        # Find the first valid extension
        match = re.search(r'\.(jpg|jpeg|png)', lower_name)
        if match:
            ext = match.group(0)
            # Truncate anything after the valid extension
            cleaned_name = filename[:match.end()]
            cleaned_path = os.path.join(input_dir, cleaned_name)

            # Rename the file if it has extra stuff
            original_path = os.path.join(input_dir, filename)
            if cleaned_name != filename:
                os.rename(original_path, cleaned_path)

            image_paths.append(cleaned_path)

    return image_paths

INPUT_DIR = "data/input/"

def process_job(conn, job: dict):
    print(f"[Worker] Processing job {job['id']}")
    inputs = job['input']
    input_dir = INPUT_DIR + job['id'] + "/"
    image_paths = find_images(input_dir)

    if len(image_paths) == 0:
        print(f"[Worker] No images found in {input_dir}")
        err = {
            "code": 400,
            "message": "No images found"
        }
        fail_job(conn, job['id'], err)
        return

    try:
        results = run_inference(inputs['prompt'], image_paths)
    except Exception as e:
        print(f"[Worker] Error processing job {job['id']}: {e}")
        err = {
            "code": 100,
            "message": str(e)
        }
        fail_job(conn, job['id'], err)
        try:
            send_webhook(inputs['webhook_url'], job['id'], None, err)
        except Exception as e:
            print(f"[Worker] Failed to send webhook: {e}")
        return

    finish_processing(conn, job['id'], results)

    try:
        send_webhook(inputs['webhook_url'], job['id'], results, None)
    except Exception as e:
        print(f"[Worker] Failed to send webhook: {e}")
        err = {
            "code": 200,
            "message": str(e)
        }
        fail_job(conn, job['id'], err)
        return

    complete_job(conn, job['id'])

    print(f"[Worker] Job {job['id']} completed.")
