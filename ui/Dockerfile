# Use nginx alpine as base image
FROM nginx:alpine

# Build arguments
ARG SERVER_URL=http://localhost:8080
ENV SERVER_URL=$SERVER_URL

# Install necessary packages
RUN apk add --no-cache \
    curl \
    && rm -rf /var/cache/apk/*

# Copy nginx configuration
COPY nginx-default.conf /etc/nginx/conf.d/default.conf

# Copy static files
COPY index.html /usr/share/nginx/html/
COPY config.js /usr/share/nginx/html/config.js

# Replace placeholder in config.js with build arg
RUN sed -i "s|%%SERVER_URL%%|$SERVER_URL|g" /usr/share/nginx/html/config.js

# Expose port 3000
EXPOSE 3000

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
