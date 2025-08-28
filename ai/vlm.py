import json
import ollama
import os
import re
import sys
import requests
from jobs import finish_processing, fail_job, complete_job
from logger_config import get_logger

# Setup logger
logger = get_logger("vlm")

def ensure_model_available(model_name):
    """
    Ensures a model is available, pulling it if necessary.
    Exits with code 1 if failed.
    """
    try:
        # Check if model exists locally
        models = ollama.list()
        logger.info("Available models", extra={"models": str(models)})
        available_models = [model['model'] for model in models['models']]
        
        if model_name in available_models:
            logger.info("Model already available", extra={"model_name": model_name})
            return
        
        logger.info("Model not found, pulling", extra={"model_name": model_name})
        ollama.pull(model_name)
        logger.info("Model pulled successfully", extra={"model_name": model_name})
        
    except Exception as e:
        logger.error("Failed to pull model", extra={"model_name": model_name, "error": str(e)})
        sys.exit(1)

import re

def parse_image_id(image_path):
    """
    Extracts a UUIDv4 id from an image path using regexp.
    Looks for a UUIDv4 pattern in the filename.
    Example: "/data/input/photo_photo_req_1754816076435_617bqy2_20250810_165440_617bqy2.jpg" -> "617bqy2" (if it's a UUIDv4)
    """
    import os
    filename = os.path.basename(image_path)
    # UUIDv4 regex: 8-4-4-4-12 hex digits
    match = re.search(
        r'([a-fA-F0-9]{8}-[a-fA-F0-9]{4}-4[a-fA-F0-9]{3}-[89abAB][a-fA-F0-9]{3}-[a-fA-F0-9]{12})',
        filename
    )
    if match:
        return match.group(1)
    return ""

def parse_image_timestamp(image_path):
    """
    Extracts the timestamp from an image path using regexp.
    Tries to find a pattern like "_{timestamp}_" or "_{timestamp}." in the filename.
    Example: "/data/input/photo_photo_req_1754816076435_617bqy2_20250810_165440.jpg" -> "20250810_165440"
    """
    import os
    filename = os.path.basename(image_path)
    # Try to match a timestamp pattern like 8 digits + underscore + 6 digits (e.g., 20250810_165440)
    match = re.search(r'_(?P<ts>\d{8}_\d{6})(?:_|\.|$)', filename)
    if match:
        return match.group('ts')
    return ""

def run_inference(vlm_prompt, prompt, image_paths):
    start_image = None
    end_image = None
    
    # Get model names from environment variables
    vlm_model = os.environ.get("VLM_MODEL", "llava:7b")
    llm_model = os.environ.get("LLM_MODEL", "llama3:latest")
    
    logger.info("Using VLM model", extra={"vlm_model": vlm_model})
    logger.info("Using LLM model", extra={"llm_model": llm_model})
    
    # Ensure models are available
    ensure_model_available(vlm_model)
    ensure_model_available(llm_model)

    logger.info("Running inference", extra={"image_count": len(image_paths), "image_paths": image_paths})
    results = "id,timestamp,event\n"

    for image_path in image_paths:
        logger.info("Processing image", extra={"image_path": image_path})
        res = ollama.chat(
            model=vlm_model,
            messages=[
                {"role": "user", "content": vlm_prompt, "images": [image_path]}
            ],
        )
        results += '"' + parse_image_id(image_path) + '",' + '"' + parse_image_timestamp(image_path) + '",' + '"' + res.message.content + '"\n'
        logger.info("Inference output", extra={"output": str(res)})

    logger.info("Inference completed")

    # Compose a prompt for temporal reasoning
    temporal_prompt = (
        "Given the timeline in the format of id,timestamp,event" + "\n" +
        "Timeline:" + results + "\n" +
        prompt
    )

    # Run the LLM for temporal reasoning
    temporal_res = ollama.chat(
        model=llm_model,
        messages=[
            {"role": "user", "content": temporal_prompt}
        ],
    )
    logger.info("Temporal reasoning output", extra={"output": temporal_res.message.content})

    return {
        "logs": results,
        "temporal_output": temporal_res.message.content
    }

def send_webhook(webhook_url, job_id, data, error):
    if webhook_url is None or webhook_url == "":
        logger.warning("Webhook URL is empty")
        return

    payload = {
        "job_id": job_id,
        "data": data,
        "error": error
    }
    try:
        response = requests.post(webhook_url, json=payload, timeout=10)
        if response.status_code != 200:
            logger.error("Failed to send webhook", extra={"status_code": response.status_code, "response": response.text})
            raise Exception(f"Failed to send webhook: {response.text}")
        else:
            logger.info("Webhook sent successfully")
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

def run(conn, job, input_dir, output_dir):
    inputs = job['input']
    image_paths = find_images(input_dir)

    if len(image_paths) == 0:
        logger.warning("No images found", extra={"input_dir": input_dir})
        err = {
            "code": 400,
            "message": "No images found"
        }
        fail_job(conn, job['id'], err)
        return

    try:
        results = run_inference(inputs['vlm_prompt'], inputs['prompt'], image_paths)
    except Exception as e:
        logger.error("Error processing job", extra={"job_id": job['id'], "error": str(e)})
        err = {
            "code": 100,
            "message": str(e)
        }
        fail_job(conn, job['id'], err)
        try:
            send_webhook(inputs['webhook_url'], job['id'], None, err)
        except Exception as e:
            logger.error("Failed to send webhook", extra={"job_id": job['id'], "error": str(e)})
        return

    finish_processing(conn, job['id'], results)

    try:
        send_webhook(inputs['webhook_url'], job['id'], results, None)
    except Exception as e:
        logger.error("Failed to send webhook", extra={"job_id": job['id'], "error": str(e)})
        err = {
            "code": 200,
            "message": str(e)
        }
        fail_job(conn, job['id'], err)
        return

    complete_job(conn, job['id'])
