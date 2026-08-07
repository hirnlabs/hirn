---
title: "server"
description: "Efficient high-scale distributed model serving across devices."
lead: "High-scale model serving infrastructure. The distributed RPC server executes inference efficiently across local hardware and networked devices."
icon: "server"
highlights:
  - title: "High Throughput"
    description: "Optimized for sustained multi-request serving with efficient caching, batching and backend-specific execution paths."
  - title: "Cross-Platform"
    description: "Natively optimized for Windows, Linux, and macOS while still scaling across heterogeneous hardware."
  - title: "Auto-Scaling"
    description: "Dynamically assigns and reclaims VRAM/resources so large models remain practical in real local deployments."
---

## Serving infrastructure, not routing logic

**hirn_server** focuses on fast and reliable model execution at scale. It does not predict intent; it serves the requests selected by router policy.

The server exposes a unified RPC interface and can target multiple backends:

- **llama.cpp:** For highly optimized CPU/GPU offloading on consumer hardware.
- **vLLM:** For high-throughput serving on larger VRAM setups.
- **exo / cactus:** For experimental and specialized local routing architectures.

## Distributed across devices

Running large models often requires more VRAM than a single machine can provide. The server pools capacity across multiple GPUs and multiple devices, turning your local network into one serving fabric.

## Operationally simple

The goal is high-scale serving without heavy ops burden: consistent RPC contracts, backend-agnostic execution, and predictable resource behavior under load.

In short: router decides *what* should run, server decides *how* to serve it efficiently across available hardware.
