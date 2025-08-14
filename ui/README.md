# Compute Node UI

A modern web UI for the Compute Node job management system, built with Vite and ES modules.

## Features

- Job submission and management
- Real-time job status updates
- Retry failed jobs
- View detailed job information
- Modern, responsive design
- Support for Rust/WASM packages

## Prerequisites

- Node.js 16.0.0 or higher
- npm 8.0.0 or higher

## Installation

1. Navigate to the UI directory:

   ```bash
   cd ui
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

## Development

Start the development server:

```bash
npm run dev
```

The application will be available at `http://localhost:3000`

## Building for Production

Build the application:

```bash
npm run build
```

Preview the production build:

```bash
npm run preview
```

## Adding Rust/WASM Packages

This project is configured to work with Rust/WASM packages. To add a WASM package:

1. Install the package:

   ```bash
   npm install @your-org/rust-wasm-package
   ```

2. Import and use in your JavaScript:

   ```javascript
   import { init, yourFunction } from '@your-org/rust-wasm-package';

   // Initialize the WASM module
   await init();

   // Use WASM functions
   const result = yourFunction(data);
   ```

3. The Vite configuration includes WASM support with:
   - `assetsInclude: ['**/*.wasm']` for WASM file handling
   - `experimental: { asyncWebAssembly: true }` for async WASM loading

## Project Structure

```
ui/
├── src/
│   ├── js/
│   │   ├── config.js          # Configuration and environment detection
│   │   ├── jobManager.js      # Job API operations
│   │   ├── uiManager.js       # UI state and DOM manipulation
│   │   └── main.js           # Application entry point
│   └── css/
│       └── styles.css        # Application styles
├── index.html                # Main HTML template
├── package.json              # Dependencies and scripts
├── vite.config.js           # Vite configuration
└── README.md                # This file
```

## Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run lint` - Run ESLint
- `npm run format` - Format code with Prettier

## Configuration

The application automatically detects the server URL based on the environment:

- Development: `http://localhost:8080`
- Production: `${currentUrl}/api/v1`

You can override this by setting the `VITE_SERVER_URL` environment variable.

## Browser Support

This application uses modern JavaScript features and ES modules. It supports:

- Chrome 80+
- Firefox 75+
- Safari 13.1+
- Edge 80+

## Troubleshooting

### WASM Loading Issues

- Ensure your WASM package is properly built and exported
- Check that the WASM file is included in the build output
- Verify browser support for WebAssembly

### Build Issues

- Clear the `dist/` directory: `rm -rf dist/`
- Clear npm cache: `npm cache clean --force`
- Reinstall dependencies: `rm -rf node_modules && npm install`
