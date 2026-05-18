<p align="center">
  <img src="docs/icon.png" width="128" height="128" alt="Nootle icon" />
</p>

<h1 align="center">Nootle</h1>

<p align="center">
  <strong>Your AI meeting recorder and assistant</strong>
  <br />
  Record, transcribe, and understand your meetings with local AI.
</p>

<p align="center">
  <a href="https://github.com/michellemayes/nootle/releases"><img src="https://img.shields.io/github/v/release/michellemayes/nootle?style=flat-square&color=72937A" alt="Release" /></a>
  <a href="https://github.com/michellemayes/nootle/actions"><img src="https://img.shields.io/github/actions/workflow/status/michellemayes/nootle/ci.yml?style=flat-square&label=CI" alt="CI" /></a>
  <a href="https://github.com/michellemayes/nootle/blob/main/LICENSE"><img src="https://img.shields.io/github/license/michellemayes/nootle?style=flat-square&color=D9B78B" alt="License" /></a>
  <img src="https://img.shields.io/badge/platform-macOS%2014%2B%20(Apple%20Silicon%20M1%E2%80%93M5)-CC765B?style=flat-square" alt="macOS 14+ (Apple Silicon M1–M5)" />
</p>

---

Nootle captures your meetings — microphone and system audio — transcribes them in real time with speaker identification, and lets you chat with an AI about what was said. No cloud recording service needed.

> **A note on privacy.** Recording and transcription are always local — meeting audio is never uploaded, and transcripts live only on your Mac. AI summaries and chat use whichever LLM provider you point Nootle at: pick **Ollama** (with a local model) to keep everything on-device, or use OpenAI, Anthropic, Google, Groq, OpenRouter, or AWS Bedrock with your own API key. You can also route through the **Claude Code CLI** (`claude -p`) or **Codex CLI** to reuse an existing Claude or ChatGPT subscription instead of an API key. Nootle is fully local end-to-end only when you use a local AI model.

## Features

- **Record everything** — capture microphone and system audio simultaneously
- **Live transcription** — speech-to-text powered by Parakeet via ONNX Runtime
- **Speaker identification** — know who said what with automatic diarization
- **AI summaries and chat** — ask questions about your meetings using your preferred LLM
- **Insight extraction** — automatically extract decisions, action items, and key moments
- **Multiple LLM providers** — OpenAI, Anthropic, Google, Groq, OpenRouter, AWS Bedrock, local Ollama, or your existing Claude / ChatGPT subscription via the Claude Code (`claude -p`) and Codex CLIs
- **Meeting detection** — auto-detects active meeting apps and calendar events
- **Workflows & integrations** — push summaries, action items, and notes to Slack, Notion, Confluence, Linear, GitHub, Asana, Obsidian, or email
- **Templates** — customizable summary templates with auto-apply rules (e.g. run a specific template for 1:1s, standups, or interviews)
- **Semantic search** — ask questions across your entire meeting history
- **Noise cancellation** — built-in denoising for cleaner audio and transcripts
- **MCP server** — integrate with Claude Code and other MCP-compatible tools
- **CLI tool** — query meetings, transcripts, insights, and more from the terminal
- **Auto-titling** — meetings are automatically titled from transcript content

## Install

Download the latest `.dmg` from [**Releases**](https://github.com/michellemayes/nootle/releases), open it, and drag Nootle to Applications.

Nootle supports Apple Silicon Macs only (M1, M2, M3, M4, and M5). Intel Macs are not supported. Download `Nootle_x.y.z_aarch64.dmg`.

### Permissions

On first launch, Nootle will ask for:

- **Microphone** — to record your voice
- **Screen Recording** — to capture system audio from meeting apps via Core Audio
- **Calendar** — to auto-detect upcoming meetings

## Development

```bash
# Install dependencies
pnpm install

# Run in dev mode
pnpm tauri dev
```

## CLI Tool

`nootle-cli` is a standalone command-line tool for querying your meeting data. It reads the Nootle database directly — the app doesn't need to be running.

```bash
# Build and install
cargo install --path src-tauri --bin nootle-cli

# List meetings
nootle-cli meetings list

# Search transcripts
nootle-cli search "quarterly review"

# List open action items
nootle-cli actions list --status open
```

Output is JSON by default. Add `--pretty` for human-readable formatting. See `nootle-cli --help` for all commands.

### Claude Code Skill

Install the skill so Claude can query your meetings:

```bash
claude skill add --global --file "$(dirname $(which nootle-cli))/../skills/nootle-cli.md"
```

## Testing

```bash
# Rust tests
cd src-tauri && cargo test
```

## Built With

- [Tauri 2](https://tauri.app) — native app shell
- [React 19](https://react.dev) — frontend UI
- [ONNX Runtime](https://onnxruntime.ai) — local ML inference
- [Tailwind CSS](https://tailwindcss.com) — styling

## License

[MIT](LICENSE)
