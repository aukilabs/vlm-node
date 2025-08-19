// add.js - Add Images Page Functionality

import { JobManager } from './jobManager.js';
import { UIManager } from './uiManager.js';
import { DownloadQuery, signInWithAppCredential } from "posemesh-domain-http";

// Initialize managers
const jobManager = new JobManager();
const uiManager = new UIManager();

// Global state
let images = [];
let selectedImages = new Set();
let authToken = null;
let lastCredentials = null;
let domainClient = null;

// DOM elements
const domainIdInput = document.getElementById('domain-id');
const appKeyInput = document.getElementById('app-key');
const appSecretInput = document.getElementById('app-secret');
const downloadBtn = document.getElementById('download-btn');
const downloadStatus = document.getElementById('download-status');
const imagesContainer = document.getElementById('images-container');
const imagesLoading = document.getElementById('images-loading');
const noImages = document.getElementById('no-images');
const selectAllCheckbox = document.getElementById('select-all-checkbox');
const submitJobBtn = document.getElementById('submit-job-btn');

// Initialize page
document.addEventListener('DOMContentLoaded', function() {
    // Load saved credentials if available
    loadSavedCredentials(); 
    // Update UI state
    updateUIState();
});

// Load saved credentials from localStorage
function loadSavedCredentials() {
    const savedDomainId = localStorage.getItem('posemesh_domain_id');
    const savedAppKey = localStorage.getItem('posemesh_app_key');
    
    if (savedDomainId) domainIdInput.value = savedDomainId;
    if (savedAppKey) appKeyInput.value = savedAppKey;
}

// Save credentials to localStorage
function saveCredentials() {
    localStorage.setItem('posemesh_domain_id', domainIdInput.value);
    localStorage.setItem('posemesh_app_key', appKeyInput.value);
}

// Download images from posemesh-domain-http
async function downloadImages() {
    const domainId = domainIdInput.value.trim();
    const appKey = appKeyInput.value.trim();
    const appSecret = appSecretInput.value.trim();
    
    if (!domainId || !appKey || !appSecret) {
        showDownloadStatus('Please fill in all fields', 'error');
        return;
    }
    
    // Check if credentials have changed
    const currentCredentials = `${appKey}:${appSecret}`;
    const needsNewLogin = lastCredentials !== currentCredentials;
    
    try {
        // downloadBtn.disabled = true;
        showDownloadStatus('Downloading images...', 'info');
        
        if (needsNewLogin) {
            showDownloadStatus('Logging in...', 'info');
            domainClient = await signInWithAppCredential("https://api.auki.network", "https://dds.auki.network", "vlm-ui", appKey, appSecret);
            lastCredentials = currentCredentials;
            saveCredentials();
        }
        
        showDownloadStatus('Fetching image list...', 'info');
        const query = new DownloadQuery([], null, null);
        console.log(domainClient);
        let imageList = [];
        await domainClient.downloadDomainData(domainId, query, function (data) {
            console.log(data);
            imageList.push(data);
        });
        
        if (imageList && imageList.length > 0) {
            images = imageList;
            renderImages();
            showDownloadStatus(`Successfully downloaded ${images.length} images`, 'success');
        } else {
            showDownloadStatus('No images found for this domain', 'info');
        }
        
    } catch (error) {
        console.error('Download failed:', error);
        showDownloadStatus(`Download failed: ${error.message}`, 'error');
        
        // Clear auth token on error
        authToken = null;
        lastCredentials = null;
    } finally {
        downloadBtn.disabled = false;
    }
}

// Login to posemesh-domain-http
async function loginToPosemesh(domainId, appKey, appSecret) {
    try {
        // Replace with your actual posemesh login endpoint
        const response = await fetch('/api/posemesh/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                domain_id: domainId,
                app_key: appKey,
                app_secret: appSecret
            })
        });
        
        if (!response.ok) {
            throw new Error(`Login failed: ${response.statusText}`);
        }
        
        const result = await response.json();
        authToken = result.token || result.access_token;
        
        if (!authToken) {
            throw new Error('No authentication token received');
        }
        
    } catch (error) {
        throw new Error(`Authentication failed: ${error.message}`);
    }
}

// Fetch image list from posemesh-domain-http
async function fetchImageList(domainId) {
    try {
        // Replace with your actual posemesh images endpoint
        const response = await fetch(`/api/posemesh/domains/${domainId}/images`, {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json',
            }
        });
        
        if (!response.ok) {
            throw new Error(`Failed to fetch images: ${response.statusText}`);
        }
        
        const result = await response.json();
        return result.images || result.data || [];
        
    } catch (error) {
        throw new Error(`Failed to fetch image list: ${error.message}`);
    }
}

// Render images in the container
function renderImages() {
    if (images.length === 0) {
        imagesLoading.style.display = 'none';
        noImages.style.display = 'block';
        imagesContainer.innerHTML = '';
        return;
    }
    
    imagesLoading.style.display = 'none';
    noImages.style.display = 'none';
    
    imagesContainer.innerHTML = '';
    
    images.forEach((image, index) => {
        const imageElement = createImageElement(image, index);
        imagesContainer.appendChild(imageElement);
    });
    
    updateUIState();
}

// Create individual image element
function createImageElement(image, index) {
    const div = document.createElement('div');
    div.className = 'image-item';
    div.dataset.index = index;
    
    if (selectedImages.has(index)) {
        div.classList.add('selected');
    }
    
    div.innerHTML = `
        <input type="checkbox" 
               class="image-checkbox" 
               ${selectedImages.has(index) ? 'checked' : ''}
               onchange="toggleImageSelection(${index}, this.checked)">
        <div class="image-name" title="${image.name || image.id}">
            ${image.name || image.id}
        </div>
        <div class="image-meta">
            ${formatFileSize(image.size || 0)}
        </div>
    `;
    
    return div;
}

// Toggle image selection
function toggleImageSelection(index, isSelected) {
    if (isSelected) {
        selectedImages.add(index);
    } else {
        selectedImages.delete(index);
    }
    
    // Update visual state
    const imageElement = document.querySelector(`[data-index="${index}"]`);
    if (imageElement) {
        imageElement.classList.toggle('selected', isSelected);
    }
    
    updateUIState();
}

// Toggle select all images
function toggleSelectAll() {
    const isSelectAll = selectAllCheckbox.checked;
    
    if (isSelectAll) {
        // Select all images
        for (let i = 0; i < images.length; i++) {
            selectedImages.add(i);
        }
    } else {
        // Deselect all images
        selectedImages.clear();
    }
    
    // Update visual state
    renderImages();
}

// Update UI state based on current selection
function updateUIState() {
    const hasImages = images.length > 0;
    const hasSelection = selectedImages.size > 0;
    
    // Update select all checkbox
    selectAllCheckbox.checked = hasImages && selectedImages.size === images.length;
    selectAllCheckbox.indeterminate = hasImages && selectedImages.size > 0 && selectedImages.size < images.length;
    
    // Update submit job button
    submitJobBtn.disabled = !hasSelection;
    
    // Update button text
    if (hasSelection) {
        submitJobBtn.textContent = `Submit Job with ${selectedImages.size} Selected Images`;
    } else {
        submitJobBtn.textContent = 'Submit Job with Selected Images';
    }
}

// Submit job with selected images
function submitJobWithSelected() {
    if (selectedImages.size === 0) {
        alert('Please select at least one image');
        return;
    }
    
    // Get selected image IDs
    const selectedImageIds = Array.from(selectedImages).map(index => {
        const image = images[index];
        return image.id || image.name;
    });
    
    // Pre-fill the submit job dialog
    const imageIdsTextarea = document.getElementById('submit-image-ids');
    const domainIdField = document.getElementById('submit-domain-id');
    
    if (imageIdsTextarea) {
        imageIdsTextarea.value = selectedImageIds.join('\n');
    }
    
    if (domainIdField) {
        domainIdField.value = domainIdInput.value;
    }
    
    // Open the submit dialog
    openSubmitDialog();
}

// Show download status message
function showDownloadStatus(message, type = 'info') {
    downloadStatus.textContent = message;
    downloadStatus.className = `status-message ${type}`;
}

// Format file size
function formatFileSize(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

// Dialog functions
function openSubmitDialog() {
    document.getElementById('submit-dialog-overlay').style.display = 'flex';
}

function closeSubmitDialog(closeAll = false) {
    if (closeAll) {
        document.getElementById('submit-dialog-overlay').style.display = 'none';
    }
}

// Submit job function
async function submitJob() {
    const vlmPrompt = document.getElementById('submit-vlm-prompt')?.value?.trim();
    const prompt = document.getElementById('submit-prompt')?.value?.trim();
    const imageIds = document.getElementById('submit-image-ids')?.value?.trim();
    const webhookUrl = document.getElementById('submit-webhook-url')?.value?.trim();
    const domainId = document.getElementById('submit-domain-id')?.value?.trim();

    if (!vlmPrompt || !imageIds || !domainId) {
        showSubmitStatus('Please fill in all required fields', 'error');
        return;
    }

    const jobData = jobManager.createJobData(
        vlmPrompt,
        prompt,
        imageIds,
        webhookUrl,
        domainId
    );

    try {
        await jobManager.submitJob(jobData);
        showSubmitStatus('Job submitted successfully!', 'success');
        setTimeout(() => {
            closeSubmitDialog(false);
            // Clear selection after successful submission
            selectedImages.clear();
            updateUIState();
        }, 1000);
    } catch (error) {
        showSubmitStatus(error.message, 'error');
    }
}

// Show submit status
function showSubmitStatus(message, type = 'info') {
    const statusElement = document.getElementById('submit-status-message');
    if (statusElement) {
        statusElement.textContent = message;
        statusElement.className = `status-message ${type}`;
    }
}

// Make functions globally available
window.downloadImages = downloadImages;
window.toggleSelectAll = toggleSelectAll;
window.toggleImageSelection = toggleImageSelection;
window.submitJobWithSelected = submitJobWithSelected;
window.openSubmitDialog = openSubmitDialog;
window.closeSubmitDialog = closeSubmitDialog;
window.submitJob = submitJob;
