#!/bin/bash
# start.sh - Startup script for vlm-node services

set -e

echo "Starting vlm-node services..."

# Function to handle cleanup on exit
cleanup() {
    echo "Shutting down services..."
    
    # Kill all background processes
    if [ ! -z "$OLLAMA_PID" ]; then
        echo "Stopping Ollama (PID: $OLLAMA_PID)..."
        kill -TERM $OLLAMA_PID 2>/dev/null || true
    fi
    
    if [ ! -z "$PYTHON_PID" ]; then
        echo "Stopping Python worker (PID: $PYTHON_PID)..."
        kill -TERM $PYTHON_PID 2>/dev/null || true
    fi
    
    if [ ! -z "$SERVER_PID" ]; then
        echo "Stopping Rust server (PID: $SERVER_PID)..."
        kill -TERM $SERVER_PID 2>/dev/null || true
    fi
    
    # Wait for processes to finish
    wait 2>/dev/null || true
    
    echo "All services stopped."
    exit 0
}

# Set up signal handlers
trap cleanup TERM INT

# Start Ollama service
echo "Starting Ollama service..."
ollama serve &
OLLAMA_PID=$!
echo "Ollama started with PID: $OLLAMA_PID"

# Wait for Ollama to be ready
echo "Waiting for Ollama to be ready..."
sleep 5

# Start Python worker
echo "Starting Python worker..."
python main.py &
PYTHON_PID=$!
echo "Python worker started with PID: $PYTHON_PID"

# Start Rust server
echo "Starting Rust server..."
/app/server &
SERVER_PID=$!
echo "Rust server started with PID: $SERVER_PID"

echo "All services started successfully."
echo "Ollama PID: $OLLAMA_PID"
echo "Python Worker PID: $PYTHON_PID"
echo "Rust Server PID: $SERVER_PID"

# Wait for any process to exit
wait
