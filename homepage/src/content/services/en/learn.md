---
title: "learn"
description: "Local finetuning loop for your personal router."
lead: "Your router shouldn't use generic rules; it should learn your specific workflow. hirn_learn provides the tooling to collect, label, and finetune your local classifier."
icon: "learn"
highlights:
  - title: "Continuous Learning"
    description: "The more you use hirn, the smarter the router gets at predicting exactly how much compute your specific prompts require."
  - title: "Privacy First"
    description: "All finetuning data stays locally on your disk. You never leak your prompts, preferences, or codebase to external endpoints."
  - title: "One-Click Finetune"
    description: "Quickly train your classifier natively in the UI with a single click, instantly deploying the updated model weights locally."
---

## Data Collection

As you work, the router silently logs every interaction locally. These logs are stored strictly on your local disk at `~/.hirn/logs/` and are never uploaded anywhere.

```json
{"input": "Refactor data layer", "intent": "coding", "files": 12, "chosen_model": "gemini-3.5-flash"}
{"input": "Debug memory leak", "intent": "debugging", "files": 3, "chosen_model": "fable-5"}
```

## The Training Loop

Using a function-calling optimized tiny model like **Needle**, you can quickly adjust the router's behavior to match your personal preferences.

Fire up the local **hirn UI** to review your historical logs, override misroutes (e.g. telling the router that specific types of PR reviews should always use Fable 5), and build a high-quality personal dataset.

Once you have a curated dataset, a one-click finetuning process teaches Needle to map your specific inputs to the correct routing function call, continuously optimizing your token spend and latency over time.
