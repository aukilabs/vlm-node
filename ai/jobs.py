import json

def get_next_job(conn):
    with conn.cursor() as cur:
        cur.execute("""
            SELECT id, input FROM jobs
            WHERE job_status = 'pending'
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE SKIP LOCKED
        """)
        job = cur.fetchone()
        if job:
            job = {
                "id": job[0],
                "input": job[1]
            }
            cur.execute("UPDATE jobs SET job_status='running', updated_at=now() WHERE id=%s", (job['id'],))
            conn.commit()
            return job
        return None

# Finish processing
def finish_processing(conn, job_id, output):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='completing', output=%s, updated_at=now()
            WHERE id=%s
        """, (json.dumps(output), job_id))
        conn.commit()

# Cancel job
def cancel_job(conn, job_id):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='cancelled', updated_at=now()
            WHERE id=%s AND job_status!='completed' AND job_status!='failed' AND job_status!='cancelled'
        """, (job_id,))
        conn.commit()

# Fail job
def fail_job(conn, job_id, error):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='failed', error=%s, updated_at=now()
            WHERE id=%s
        """, (json.dumps(error), job_id))
        conn.commit()

# Complete job
def complete_job(conn, job_id):
    with conn.cursor() as cur:
        cur.execute("""
            UPDATE jobs
            SET job_status='completed', updated_at=now()
            WHERE id=%s
        """, (job_id,))
        conn.commit()

# Upload job output to domain server
def upload_job_result(conn, job_id, output=None):
    if output is None:
        with conn.cursor() as cur:
            cur.execute("""
                UPDATE jobs
                SET job_status='uploading', updated_at=now()
                WHERE id=%s
            """, (job_id,))
            conn.commit()
    else:
        with conn.cursor() as cur:
            cur.execute("""
                UPDATE jobs
                SET job_status='uploading', updated_at=now(), output=%s
                WHERE id=%s
            """, (json.dumps(output), job_id))
