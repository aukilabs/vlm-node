import logging
import json
import sys
from datetime import datetime
from typing import Any, Dict

class JSONFormatter(logging.Formatter):
    """Custom JSON formatter for structured logging"""
    
    def format(self, record: logging.LogRecord) -> str:
        log_entry = {
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
            "module": record.module,
            "function": record.funcName,
            "line": record.lineno
        }
        
        # Add extra fields if they exist
        if hasattr(record, 'job_id'):
            log_entry['job_id'] = record.job_id
        if hasattr(record, 'worker_id'):
            log_entry['worker_id'] = record.worker_id
        if hasattr(record, 'dispatcher_id'):
            log_entry['dispatcher_id'] = record.dispatcher_id
        if hasattr(record, 'domain_id'):
            log_entry['domain_id'] = record.domain_id
        if hasattr(record, 'error'):
            log_entry['error'] = record.error
        # Add exception info if present
        if record.exc_info:
            log_entry['exception'] = self.formatException(record.exc_info)
            
        return json.dumps(log_entry)

def setup_logger(name: str, level: str = "INFO") -> logging.Logger:
    """Setup a logger with JSON formatting"""
    logger = logging.getLogger(name)
    logger.setLevel(getattr(logging, level.upper()))
    
    # Remove existing handlers to avoid duplicates
    for handler in logger.handlers[:]:
        logger.removeHandler(handler)
    
    # Create console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(logging.DEBUG)
    
    # Set JSON formatter
    formatter = JSONFormatter()
    console_handler.setFormatter(formatter)
    
    # Add handler to logger
    logger.addHandler(console_handler)
    
    # Prevent propagation to root logger
    logger.propagate = False
    
    return logger

def get_logger(name: str) -> logging.Logger:
    """Get a logger instance with the given name, auto-setup if not configured"""
    logger = logging.getLogger(name)
    
    # If logger has no handlers, set it up automatically
    if not logger.handlers:
        setup_logger(name)
    
    return logger

# Setup default logging configuration
def setup_default_logging():
    """Setup default logging configuration for the application"""
    # Set up root logger to avoid any unhandled log messages
    root_logger = logging.getLogger()
    root_logger.setLevel(logging.WARNING)
    
    # Set up console handler for root logger
    console_handler = logging.StreamHandler(sys.stderr)
    console_handler.setLevel(logging.WARNING)
    formatter = JSONFormatter()
    console_handler.setFormatter(formatter)
    root_logger.addHandler(console_handler)
