# worker.py
import json
import torch
from llava.model.builder import load_pretrained_model
from llava.serve.cli import eval_model

MODEL_PATH = "liuhaotian/llava-v1.5-7b"
device = torch.device("mps" if torch.backends.mps.is_available() else "cpu")

print("[Worker] Loading LLaVA model (one-time init)...")
tokenizer, model, image_processor, context_len = load_pretrained_model(
    model_path=MODEL_PATH,
    model_base=None,
    model_name=MODEL_PATH,
    device=device
)

def fetch_job(conn, job_id):
    with conn.cursor() as cur:
        cur.execute("SELECT prompt, input_images FROM jobs WHERE id=%s", (job_id,))
        return cur.fetchone()

def update_job_output(conn, job_id, output):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET status='process done', output=%s
            WHERE id=%s
        """, (json.dumps(output), job_id))
        conn.commit()

def run_inference(prompt, image_paths):
    outputs = []
    for img_path in image_paths:
        resp = eval_model(
            model_path=MODEL_PATH,
            model_base=None,
            model_name=MODEL_PATH,
            tokenizer=tokenizer,
            model=model,
            image_processor=image_processor,
            context_len=context_len,
            image_file=img_path,
            query=prompt,
            device=device
        )
        outputs.append({"image": img_path, "description": resp})
    return outputs

def process_job(conn, job_id: int):
    print(f"[Worker] Processing job {job_id}")
    prompt, input_images = fetch_job(conn, job_id)
    image_paths = json.loads(input_images)

    results = run_inference(prompt, image_paths)
    update_job_output(conn, job_id, results)
    print(f"[Worker] Job {job_id} completed.")
