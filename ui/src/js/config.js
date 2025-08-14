// Configuration file for the application
export const APP_CONFIG = {
  serverUrl: import.meta.env.VITE_SERVER_URL || 'http://localhost:8080',
};

// Auto-detect web URL and set API endpoint
(function () {
  // Get current web URL
  const currentUrl = window.location.origin;

  // Check if we're in development mode (localhost)
  if (currentUrl.includes('localhost') || currentUrl.includes('127.0.0.1')) {
    // Development: use localhost:8080
    APP_CONFIG.serverUrl = 'http://localhost:8080/api/v1';
  } else {
    // Production: use current web URL + /api
    APP_CONFIG.serverUrl = `${currentUrl}/api/v1`;
  }

  console.log('Auto-detected server URL:', APP_CONFIG.serverUrl);
  console.log('Current web URL:', currentUrl);

  // Log final configuration
  console.log('Final APP_CONFIG:', APP_CONFIG);
})();

export default APP_CONFIG;
