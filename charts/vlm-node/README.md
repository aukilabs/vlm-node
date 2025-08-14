# Compute Node Helm Chart

This Helm chart deploys a compute node with separate server and worker components that can scale independently. The chart is designed to be a template for different types of compute nodes such as SLAM, task-timing, etc.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              External Users                                 │
└─────────────────────┬─────────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Ingress Controller                            │
│                              (nginx)                                       │
└─────────────────────┬─────────────────────────────────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        │             │             │
        ▼             ▼             ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│    UI       │ │   Server    │ │   Worker    │
│   Port 3000 │ │  Port 8080  │ │  Port 8080  │
│             │ │             │ │             │
│  • Serves   │ │  • Handles  │ │  • Processes│
│    static   │ │    HTTP     │ │    jobs     │
│    files    │ │    requests │ │  • Reads    │
│  • Talks to │ │  • Creates  │ │    from DB  │
│    server   │ │    jobs in  │ │  • Writes   │
│    via      │ │    DB       │ │    to DB    │
│    external │ │  • Reads    │ │  • Accesses │
│    URL      │ │    from DB  │ │    shared   │
│             │ │  • Writes   │ │    to DB    │
│             │ │             │ │             │
└─────────────┘ └─────────────┘ └─────────────┘
                      │             │
                      └─────────────┼─────────┐
                                    │         │
                                    ▼         ▼
                        ┌─────────────────────────────────┐
                        │         PostgreSQL              │
                        │      (Bitnami Chart)           │
                        │         Port 5432               │
                        │                                 │
                        │  • Job management               │
                        │  • Data persistence             │
                        │  • Communication hub            │
                        └─────────────────────────────────┘
                                    │
                                    ▼
                        ┌─────────────────────────────────┐
                        │       Shared Storage            │
                        │      (Persistent Volume)       │
                        │                                 │
                        │  • File uploads (Server)        │
                        │  • File processing (Worker)     │
                        │  • Shared filesystem access     │
                        └─────────────────────────────────┘
```

### Component Communication Flow

1. **UI → Server**: UI communicates with server via external URL (not internal service)
2. **Server ↔ Database**: Server creates jobs and reads/writes data
3. **Worker ↔ Database**: Worker polls for jobs and updates status
4. **Server ↔ Worker**: Communication happens through database changes
5. **Storage Sharing**: Both server and worker access the same persistent volume

## Features

- StatefulSets for both server and worker for stable storage access
- Cloud-agnostic persistent storage with configurable storage classes
- Built-in Bitnami PostgreSQL database with persistent storage (via Helm dependency)
- Configurable scaling for server and worker independently
- Ingress configuration for public access
- Flexible resource naming using chart name or custom node name override
- UI served by nginx on port 3000
- Range-based environment variable configuration
- Component-specific Docker images

## Prerequisites

- Kubernetes cluster with persistent storage support
- Helm 3.x
- Container registry with your images

## Dependencies

This chart depends on the [Bitnami PostgreSQL chart](https://artifacthub.io/packages/helm/bitnami/postgresql) which will be automatically installed when `postgresql.enabled` is set to `true`.

## Storage Strategy

### Current Approach: Shared Persistent Volume (Required)
- **Single PVC**: Both server and worker mount the same persistent volume
- **Shared Access**: Files uploaded by server are immediately available to worker
- **Pros**: Simple, efficient file sharing, no data duplication, enables server-worker communication
- **Cons**: Single point of failure, can't scale storage independently

**⚠️ Important**: This shared storage approach is **required** for your architecture. The server downloads files that the worker must access, which is only possible with shared storage.
