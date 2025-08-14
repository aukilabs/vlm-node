import { JobManager } from './jobManager.js';
import { UIManager } from './uiManager.js';
import '../css/styles.css';

// Initialize managers
window.jobManager = new JobManager();
window.uiManager = new UIManager();

// Global functions for onclick handlers
window.openSubmitDialog = () => window.uiManager.openSubmitDialog();
window.openRetryDialog = (jobId) => window.uiManager.openRetryDialog(jobId);
window.openViewDialog = (jobId) => window.uiManager.openViewDialog(jobId);

// Load jobs on page load
document.addEventListener('DOMContentLoaded', function () {
  loadJobs();
});

async function loadJobs() {
  try {
    const jobs = await window.jobManager.loadJobs();
    window.uiManager.displayJobs(jobs);
  } catch (error) {
    window.uiManager.showStatus(error.message, 'error');
  }
}

// Submit job function
async function submitJob() {
  const vlmPrompt = document.getElementById('submit-vlm-prompt')?.value?.trim();
  const prompt = document.getElementById('submit-prompt')?.value?.trim();
  const imageIds = document.getElementById('submit-image-ids')?.value?.trim();
  const webhookUrl = document
    .getElementById('submit-webhook-url')
    ?.value?.trim();
  const domainId = document.getElementById('submit-domain-id')?.value?.trim();

  if (!vlmPrompt || !imageIds || !domainId) {
    window.uiManager.showSubmitStatus(
      'Please fill in all required fields',
      'error'
    );
    return;
  }

  const jobData = window.jobManager.createJobData(
    vlmPrompt,
    prompt,
    imageIds,
    webhookUrl,
    domainId
  );

  try {
    await window.jobManager.submitJob(jobData);
    window.uiManager.showSubmitStatus('Job submitted successfully!', 'success');
    setTimeout(() => {
      window.uiManager.closeSubmitDialog(false);
      loadJobs();
    }, 1000);
  } catch (error) {
    window.uiManager.showSubmitStatus(error.message, 'error');
  }
}

// Retry job function
async function retryJob() {
  if (!window.jobManager.currentRetryJobId) return;

  const vlmPrompt = document.getElementById('retry-vlm-prompt')?.value?.trim();
  const prompt = document.getElementById('retry-prompt')?.value?.trim();
  const webhookUrl = document
    .getElementById('retry-webhook-url')
    ?.value?.trim();

  if (!vlmPrompt) {
    window.uiManager.showRetryStatus(
      'Please fill in all required fields',
      'error'
    );
    return;
  }

  const jobData = window.jobManager.createJobData(
    vlmPrompt,
    prompt,
    document.getElementById('retry-image-ids')?.value || '',
    webhookUrl,
    document.getElementById('retry-domain-id')?.value || ''
  );

  try {
    await window.jobManager.retryJob(
      window.jobManager.currentRetryJobId,
      jobData
    );
    window.uiManager.showRetryStatus('Job updated successfully!', 'success');
    setTimeout(() => {
      window.uiManager.closeRetryDialog(false);
      loadJobs();
    }, 1000);
  } catch (error) {
    window.uiManager.showRetryStatus(error.message, 'error');
  }
}

// Make functions globally available
window.submitJob = submitJob;
window.retryJob = retryJob;
