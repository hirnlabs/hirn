# Router

An intelligent, cost-saving local gateway for the Hirn system. It acts as a transparent proxy behind your client (Pi/Claude Code), optimizing model usage by classifying user intent and routing requests-prioritizing local models whenever possible.

## Key Features

- **Local-First Priority**: The router is designed to offload as much work as possible to high-performance local LLMs, reducing latency and reliance on external API rate limits.
- **Classifier Routing Architecture**: Uses a multi-step pipeline to evaluate, select, and execute the optimal model for each turn.
- **Cost Optimization**: Drastically cuts API costs by offloading routine tasks to local compute before falling back to frontier models.
- **Deterministic Selection**: Uses a tuneable weighted-sum algorithm to pick the best model (local or remote) based on latency, quality, cost, and API limits.
- **Self-Improving**: Logs request metadata to refine tier prediction and scoring weights over time.

## Architecture

As defined in `homepage/src/pages/router.astro`, the router pipeline consists of:

1.  **Lightweight Classifier**: Determines the required effort/tier, identifying whether a local model is sufficient for the task.
2.  **Deterministic Selector**: Maps tiers to candidate models (prioritizing local models if they meet quality/performance criteria).
3.  **Model Execution**: Forwards the request to the selected model registry (e.g., local models via Ollama/Llama.cpp, or Fable, Opus, Gemini).
4.  **Feedback Loop**: Logs performance data to feed back into the classifier and scoring functions.
