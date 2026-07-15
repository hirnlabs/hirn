# Server

The central backend infrastructure for Hirn, functioning as a high-performance `llama.cpp` orchestration server and load balancer.

## Core Responsibilities
- **Distributed Orchestration**: Manages multiple `llama.cpp` inference nodes across your local network. You can define multiple hosts and ports, and the server will intelligently distribute models to balance load.
- **RPC Communication**: Uses RPC to coordinate tasks between the orchestration server and distributed inference workers.
- **Model Sharding**: Enables automatic sharding of large models across multiple devices or GPUs, allowing you to run models that exceed the capacity of a single machine.
- **VRAM Pooling**: Aggregates VRAM resources from multiple machines, treating your local cluster as a unified inference compute pool.

## Architecture
- **API Gateway**: Entry point for all external client communications (Desktop/Mobile).
- **Inference Manager**: Orchestrates the startup, shutdown, and distribution of inference jobs.
- **Auto-Scaling**: Dynamically unloads idle models and shards to optimize memory usage across the cluster.
- **Cross-Platform Support**: Optimized for Linux, macOS, and Windows, ensuring bare-metal performance across your entire hardware fleet.
