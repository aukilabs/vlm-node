export class UIManager {
  constructor() {
    this.initializeEventListeners();
  }

  initializeEventListeners() {
    // Submit dialog events
    document
      .getElementById('submit-dialog-overlay')
      ?.addEventListener('click', (e) => {
        if (e.target.id === 'submit-dialog-overlay') {
          this.closeSubmitDialog(true);
        }
      });

    // Retry dialog events
    document
      .getElementById('retry-dialog-overlay')
      ?.addEventListener('click', (e) => {
        if (e.target.id === 'retry-dialog-overlay') {
          this.closeRetryDialog(true);
        }
      });

    // View dialog events
    document
      .getElementById('view-dialog-overlay')
      ?.addEventListener('click', (e) => {
        if (e.target.id === 'view-dialog-overlay') {
          this.closeViewDialog();
        }
      });
  }

  displayJobs(jobs) {
    const tbody = document.getElementById('jobs-tbody');
    if (!tbody) return;

    tbody.innerHTML = '';

    jobs.forEach((job) => {
      const row = document.createElement('tr');
      row.innerHTML = `
        <td>${job.id}</td>
        <td>${job.status || 'N/A'}</td>
        <td>${this.formatDate(job.created_at)}</td>
        <td>${this.formatDate(job.updated_at)}</td>
        <td>
          <button class="retry-btn" onclick="window.uiManager.openRetryDialog('${job.id}')" style="margin-right: 5px;">Retry</button>
          <button class="btn btn-primary" onclick="window.uiManager.openViewDialog('${job.id}')">View</button>
        </td>
      `;
      tbody.appendChild(row);
    });
  }

  formatDate(dateString) {
    if (!dateString) return 'N/A';
    try {
      return new Date(dateString).toLocaleString();
    } catch (e) {
      return dateString;
    }
  }

  showStatus(message, type) {
    const statusElement = document.getElementById('status-message');
    if (!statusElement) return;

    statusElement.textContent = message;
    statusElement.className = `status-message ${type}`;
    statusElement.style.display = 'block';

    setTimeout(() => {
      statusElement.style.display = 'none';
    }, 5000);
  }

  // Submit Dialog Methods
  openSubmitDialog() {
    const overlay = document.getElementById('submit-dialog-overlay');
    if (overlay) {
      overlay.style.display = 'flex';
      this.clearSubmitForm();
    }
  }

  closeSubmitDialog(shouldConfirm = true) {
    if (
      !shouldConfirm ||
      confirm(
        'Are you sure you want to close this dialog? Any unsaved changes will be lost.'
      )
    ) {
      const overlay = document.getElementById('submit-dialog-overlay');
      if (overlay) {
        overlay.style.display = 'none';
        this.clearSubmitForm();
      }
    }
  }

  clearSubmitForm() {
    const elements = [
      'submit-vlm-prompt',
      'submit-prompt',
      'submit-image-ids',
      'submit-webhook-url',
      'submit-domain-id',
      'submit-status-message',
    ];

    elements.forEach((id) => {
      const element = document.getElementById(id);
      if (element) {
        if (element.tagName === 'TEXTAREA' || element.tagName === 'INPUT') {
          element.value = '';
        } else {
          element.style.display = 'none';
        }
      }
    });
  }

  showSubmitStatus(message, type) {
    const statusElement = document.getElementById('submit-status-message');
    if (statusElement) {
      statusElement.textContent = message;
      statusElement.className = `status-message ${type}`;
      statusElement.style.display = 'block';
    }
  }

  // Retry Dialog Methods
  openRetryDialog(jobId) {
    window.jobManager.currentRetryJobId = jobId;
    window.jobManager
      .getJob(jobId)
      .then((job) => {
        this.populateRetryForm(job);
        const overlay = document.getElementById('retry-dialog-overlay');
        if (overlay) {
          overlay.style.display = 'flex';
        }
      })
      .catch((error) => {
        this.showStatus(`Error loading job: ${error.message}`, 'error');
      });
  }

  populateRetryForm(job) {
    const fields = {
      'retry-job-id': job.id || '',
      'retry-job-status': job.status || '',
      'retry-domain-id': job.input?.domain_id || '',
      'retry-image-ids': Array.isArray(job.input?.image_ids)
        ? job.input.image_ids.join('\n')
        : job.input?.image_ids || '',
      'retry-vlm-prompt': job.input?.vlm_prompt || '',
      'retry-prompt': job.input?.prompt || '',
      'retry-webhook-url': job.input?.webhook_url || '',
    };

    Object.entries(fields).forEach(([id, value]) => {
      const element = document.getElementById(id);
      if (element) {
        element.value = value;
      }
    });

    const statusElement = document.getElementById('retry-status-message');
    if (statusElement) {
      statusElement.style.display = 'none';
    }
  }

  closeRetryDialog(shouldConfirm = true) {
    if (
      !shouldConfirm ||
      confirm(
        'Are you sure you want to close this dialog? Any unsaved changes will be lost.'
      )
    ) {
      const overlay = document.getElementById('retry-dialog-overlay');
      if (overlay) {
        overlay.style.display = 'none';
        window.jobManager.currentRetryJobId = null;
      }
    }
  }

  showRetryStatus(message, type) {
    const statusElement = document.getElementById('retry-status-message');
    if (statusElement) {
      statusElement.textContent = message;
      statusElement.className = `status-message ${type}`;
      statusElement.style.display = 'block';
    }
  }

  // View Dialog Methods
  openViewDialog(jobId) {
    window.jobManager
      .getJob(jobId)
      .then((job) => {
        this.populateViewForm(job);
        const overlay = document.getElementById('view-dialog-overlay');
        if (overlay) {
          overlay.style.display = 'flex';
        }
      })
      .catch((error) => {
        this.showStatus(`Error loading job: ${error.message}`, 'error');
      });
  }

  populateViewForm(job) {
    const fields = {
      'view-job-id': job.id || '',
      'view-job-type': job.job_type || '',
      'view-job-status': job.status || '',
      'view-domain-id': job.input?.domain_id || '',
      'view-image-ids': Array.isArray(job.input?.image_ids)
        ? job.input.image_ids.join('\n')
        : job.input?.image_ids || '',
      'view-vlm-prompt': job.input?.vlm_prompt || '',
      'view-prompt': job.input?.prompt || '',
      'view-webhook-url': job.input?.webhook_url || '',
      'view-created-at': this.formatDate(job.created_at),
      'view-updated-at': this.formatDate(job.updated_at),
    };

    Object.entries(fields).forEach(([id, value]) => {
      const element = document.getElementById(id);
      if (element) {
        element.value = value;
      }
    });

    // Display error and output as pretty JSON if they exist
    this.displayJsonField('view-error', job.error);
    this.displayJsonField('view-output', job.output);
  }

  closeViewDialog() {
    const overlay = document.getElementById('view-dialog-overlay');
    if (overlay) {
      overlay.style.display = 'none';
    }
  }

  displayJsonField(fieldId, data) {
    const field = document.getElementById(fieldId);
    if (!field) return;

    if (data !== null && data !== undefined) {
      try {
        // If data is already a string, try to parse it
        const jsonData = typeof data === 'string' ? JSON.parse(data) : data;
        field.textContent = JSON.stringify(jsonData, null, 2);
        field.style.display = 'block';
      } catch (e) {
        // If parsing fails, display as string
        field.textContent = String(data);
        field.style.display = 'block';
      }
    } else {
      field.style.display = 'none';
    }
  }
}
