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
            SET job_status='completing', output=%s, updated_at=now()
            WHERE id=%s
        """, (json.dumps(output), job_id))
        conn.commit()

def cancel_job(conn, job_id):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='cancelled', updated_at=now()
            WHERE id=%s AND job_status!='completed' AND job_status!='failed' AND job_status!='cancelled'
        """, (job_id,))
        conn.commit()

def fail_job(conn, job_id, error):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='failed', error=%s, updated_at=now()
            WHERE id=%s
        """, (json.dumps(error), job_id))
        conn.commit()

def complete_job(conn, job_id):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='completed', updated_at=now()
            WHERE id=%s
        """, (job_id,))
        conn.commit()

def parse_image_id(image_path):
    """
    Extracts the id from an image path of the form "/xxx/xxxx/{timestamp}_{id}.{data_type}"
    Example: "/data/input/2025-08-04T10-09-49-186Z_abc123.png" -> "abc123"
    """
    import os
    filename = os.path.basename(image_path)
    if "_" in filename:
        id_part = filename.split("_")[2]
        return id_part
    return None

def parse_image_timestamp(image_path):
    """
    Extracts the timestamp from an image path of the form "/xxx/xxxx/{timestamp}_{id}.{data_type}"
    Example: "/data/input/2025-08-04T10-09-49-186Z_abc123.png" -> "2025-08-04T10-09-49-186Z"
    """
    import os
    filename = os.path.basename(image_path)
    if "_" in filename:
        timestamp_part = filename.split("_")[0]
        return timestamp_part
    return None

class TaskTiming(BaseModel):
    start_image: str
    end_image: str

def run_inference(vlm_prompt, prompt, image_paths):
    start_image = None
    end_image = None
    model = os.environ.get("MODEL", "gemma3:4b")

    print(f"[Worker] Running inference for {len(image_paths)} images: {image_paths}")
    results = "timestamp,event\n"

    for image_path in image_paths:
        print(f"[Worker] Image path: {image_path}")
        res = ollama.chat(
            model="llava:7b",
            messages=[
                {"role": "user", "content": vlm_prompt, "images": [image_path]}
            ],
        )
        results += '"' + parse_image_timestamp(image_path) + '",' + '"' + res.message.content + '"\n'
        print(f"[Worker] Inference output: {res}")

    print(f"[Worker] Inference completed")

    # Compose a prompt for temporal reasoning
    temporal_prompt = (
        "Given the timeline in the format of timestamp,event" + "\n" +
        "Timeline:" + results + "\n" +
        prompt
    )

    # Run the LLM for temporal reasoning
    temporal_res = ollama.chat(
        model="llama3",
        messages=[
            {"role": "user", "content": temporal_prompt}
        ],
    )
    print(f"[Worker] Temporal reasoning output: {temporal_res.message.content}")

    return {
        "logs": results,
        "temporal_output": temporal_res.message.content
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
    
    image_paths.sort()

    return image_paths

DATA_DIR = os.environ.get("DATA_DIR", "data")
INPUT_DIR = DATA_DIR + "/input/"

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
        results = run_inference(inputs['vlm_prompt'], inputs['prompt'], image_paths)
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
