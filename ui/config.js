// Configuration file - will be replaced by Docker build arg
window.APP_CONFIG = {
    serverUrl: '%%SERVER_URL%%'
};

// Auto-detect web URL and set API endpoint
(function() {
    // If the placeholder wasn't replaced by Docker build, auto-detect
    if (window.APP_CONFIG.serverUrl === '%%SERVER_URL%%') {
        // Get current web URL
        const currentUrl = window.location.origin;
        
        // Check if we're in development mode (localhost)
        if (currentUrl.includes('localhost') || currentUrl.includes('127.0.0.1')) {
            // Development: use localhost:8080
            window.APP_CONFIG.serverUrl = 'http://localhost:8080';
        } else {
            // Production: use current web URL + /api
            window.APP_CONFIG.serverUrl = `${currentUrl}/api/v1`;
        }
        
        console.log('Auto-detected server URL:', window.APP_CONFIG.serverUrl);
        console.log('Current web URL:', currentUrl);
    }
    
    // Log final configuration
    console.log('Final APP_CONFIG:', window.APP_CONFIG);
})();
