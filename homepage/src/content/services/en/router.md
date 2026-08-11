---
title: "router"
description: "Intent prediction model for task and model-tier selection."
lead: "An intelligent, cost-saving local model gateway. It sits transparently behind your agents. You keep your standard Pro subscriptions; the router decides which one to use per turn and if local might be the cheapest and fastest. It classifies each user turn and selects the right thinking level and model tier before execution happens."
icon: "router"
highlights:
  - title: "Cost Savings"
    description: "By offloading heavy context tasks to workhorse models, the router cuts API costs by up to 80% while preserving high-tier reasoning."
  - title: "Zero Latency"
    description: "The ~26M parameter local classifier executes in less than 50ms on device, adding virtually zero overhead to your request pipeline."
  - title: "Self-Improving"
    description: "Every correction can be fed into learn to retrain the router on your personal workflow."
---

## Intent-first routing

Instead of treating all prompts equally, **hirn_router** predicts user intent from request features and context. The output is a structured routing decision that determines what should run next.

This is where cost and quality policy lives. Router evaluates whether a request is routine, medium-depth, or reasoning-heavy, then maps it to the right capability tier before any model execution starts.

## The Problem: Opaque Pricing & Token Limits

Frontier models are expensive and plan limits are easy to hit. Large codebase prompts can burn through credits quickly, even when the task only needs moderate capability. For example, Claude Fable 5 costs $10/1M input and $50/1M output. Running large codebases through it burns through your limits and credits instantly, even for simple formatting or refactoring tasks that don't require immense reasoning.

Router prevents over-spending by predicting intent first, then applying a deterministic selection policy instead of always defaulting to the most expensive model.

## Router decision pipeline

Instead of routing everything to a frontier model, **hirn_router** uses a multi-step pipeline to evaluate, select, and execute the optimal model for each turn. The router is a prediction system, which produces deterministic decisions that downstream components execute.

1. **Intent classification:** Predicts user intent from prompt content, session context, and feature signals gathered during agent operation.
2. **Tier and path selection:** Maps the predicted intent to a task path and model tier, then scores candidates using latency, quality, cost, and limit pressure.
3. **Execution handoff:** Hands off the decision to execution components (especially **hirn_server**) to run inference on selected backends.
4. **Feedback capture for learn:** Captures outcomes and overrides so **hirn_learn** can train improved versions of the router on your own usage.

## The Math: Weighted Normalized Sum

For candidate models in the predicted tier, router applies a tuneable weighted normalized score:

```text
score = (w_q * Q) - (w_l * L) - (w_c * C) - (w_limit * U)
```

- **Q:** Normalized quality signal for the target task class.
- **L:** Normalized latency expectation.
- **C:** Expected marginal cost for the request.
- **U:** Penalty for proximity to quota/token limits.

The highest score wins deterministically. This keeps behavior transparent, debuggable, and aligned with your own latency/cost priorities. But you can also plug in the local API and run multiple high scoring ones or sample from the model pool with temperature to get more randomness.

## Trained by learn

The router improves over time via **hirn_learn**, which provides dataset curation, supervised labeling, and local fine-tuning based on your real agent sessions.

In short: router predicts intent and policy, learn trains that predictor, and server handles high-scale inference execution.
