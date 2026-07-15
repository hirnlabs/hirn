# Transcribe

The local speech-to-text processing engine for the Hirn system. It enables low-latency voice input, allowing users to interact with agents using spoken language while maintaining strict privacy.

## Core Features
- **Local Whisper Engine**: Leverages [OpenAI's Whisper](https://github.com/openai/whisper) models locally for high-accuracy, privacy-first transcription.
- **Real-time Streaming**: Provides low-latency, real-time transcription of live microphone input, optimized for coding tasks and system control.
- **Privacy First**: All audio processing happens entirely on-device; audio streams are discarded immediately after transcription to ensure zero data exfiltration.
- **VAD Optimized**: Features built-in Voice Activity Detection to intelligently manage compute resources and only process active speech.

## Architecture
- **Transcription Pipeline**: Captures raw audio, performs local inference using Whisper, and outputs structured text to the Hirn agent/router.
- **Language Detection**: Automatically detects spoken languages and can translate on-the-fly to ensure optimal input for the Agent's context window.
- **Global Integration**: Accessible via global hotkeys in the desktop client, providing a seamless "voice-to-agent" experience across the OS.
