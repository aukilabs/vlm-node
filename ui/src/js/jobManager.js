import { APP_CONFIG } from './config.js';

export class JobManager {
  constructor() {
    this.currentRetryJobId = null;
  }

  async loadJobs() {
    try {
      const response = await fetch(`${APP_CONFIG.serverUrl}/jobs?limit=100`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      if (response.ok) {
        const jobs = await response.json();
        return jobs;
      } else {
        throw new Error(`Error loading jobs: ${response.status}`);
      }
    } catch (error) {
      throw new Error(`Network error: ${error.message}`);
    }
  }

  async submitJob(jobData) {
    try {
      const response = await fetch(`${APP_CONFIG.serverUrl}/jobs`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(jobData),
      });

      if (response.ok) {
        return await response.json();
      } else {
        const errorText = await response.text();
        throw new Error(`Error: ${response.status} - ${errorText}`);
      }
    } catch (error) {
      throw new Error(`Network error: ${error.message}`);
    }
  }

  async getJob(jobId) {
    try {
      const response = await fetch(`${APP_CONFIG.serverUrl}/jobs/${jobId}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });
      if (response.ok) {
        return await response.json();
      } else {
        throw new Error(`Error loading job: ${response.status}`);
      }
    } catch (error) {
      throw new Error(`Network error: ${error.message}`);
    }
  }

  async retryJob(jobId, jobData) {
    try {
      const response = await fetch(`${APP_CONFIG.serverUrl}/jobs/${jobId}`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(jobData),
      });

      if (response.ok) {
        return await response.json();
      } else {
        const errorText = await response.text();
        throw new Error(`Error: ${response.status} - ${errorText}`);
      }
    } catch (error) {
      throw new Error(`Network error: ${error.message}`);
    }
  }

  formatDate(dateString) {
    if (!dateString) return 'N/A';
    try {
      return new Date(dateString).toLocaleString();
    } catch (e) {
      return dateString;
    }
  }

  createJobData(vlmPrompt, prompt, imageIds, webhookUrl, domainId) {
    return {
      job_type: 'task_timing_v1',
      query: {
        ids: imageIds.split('\n').filter((id) => id.trim()),
      },
      domain_id: domainId,
      input: {
        vlm_prompt: vlmPrompt,
        prompt: prompt,
        webhook_url: webhookUrl,
      },
    };
  }
}
