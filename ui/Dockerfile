# Use nginx alpine as base image
FROM nginx:alpine

# Install necessary packages
RUN apk add --no-cache \
    curl \
    && rm -rf /var/cache/apk/*

# Copy nginx configuration
COPY nginx-default.conf /etc/nginx/conf.d/default.conf

# Copy static files
COPY index.html /usr/share/nginx/html/
COPY *.js /usr/share/nginx/html/ 2>/dev/null || true
COPY *.css /usr/share/nginx/html/ 2>/dev/null || true
COPY assets/ /usr/share/nginx/html/assets/ 2>/dev/null || true

# Create a non-root user
RUN addgroup -g 1001 -S nginx && \
    adduser -S -D -H -u 1001 -h /var/cache/nginx -s /sbin/nologin -G nginx -g nginx nginx

# Change ownership of nginx directories
RUN chown -R nginx:nginx /var/cache/nginx && \
    chown -R nginx:nginx /var/log/nginx && \
    chown -R nginx:nginx /etc/nginx/conf.d && \
    chown -R nginx:nginx /usr/share/nginx/html

# Switch to non-root user
USER nginx

# Expose port 3000
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
