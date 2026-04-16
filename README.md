<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://img.shields.io/badge/Artifex-v0.5-blue?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJ3aGl0ZSI+PHBhdGggZD0iTTEyIDJMMiA3bDEwIDUgMTAtNS0xMC01ek0yIDE3bDEwIDUgMTAtNU0yIDEybDEwIDUgMTAtNSIvPjwvc3ZnPg==&labelColor=1a1a2e&color=e94560">
  <img alt="Artifex" src="https://img.shields.io/badge/Artifex-v0.5-blue?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJibGFjayI+PHBhdGggZD0iTTEyIDJMMiA3bDEwIDUgMTAtNS0xMC01ek0yIDE3bDEwIDUgMTAtNU0yIDEybDEwIDUgMTAtNSIvPjwvc3ZnPg==&labelColor=e8e8e8&color=2563eb">
</picture>

**AI-powered creative suite for game developers.** Generate sprites, audio, music, voice, and more — all from your desktop.

[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri v2](https://img.shields.io/badge/Tauri-v2-24c8db?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2-ff3e00?logo=svelte&logoColor=white)](https://kit.svelte.dev/)
[![Tests](https://img.shields.io/badge/tests-256-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-green)](./LICENSE)

---

## What is Artifex?

Artifex is a **multi-platform desktop application** that gives indie game developers a unified creative workspace powered by AI. Instead of juggling multiple subscriptions and browser tabs, you get:

- **Image generation** — sprites, tilesets, concept art, materials
- **Audio generation** — sound effects, ambient loops
- **Music generation** — background music, themes
- **Text-to-speech** — NPC dialogue, narration, voice-over
- **Prompt templates** — reusable, parameterized prompts for consistent output

All through a clean UI with provider management, credential storage via OS keychain, and real-time job progress tracking.

## Architecture

Artifex follows **Domain-Driven Design (DDD)** with a Control Plane + Data Plane architecture:

```
┌─────────────────────────────────────────────────────┐
│                   SvelteKit UI                       │
│  Projects │ Assets │ Settings │ Generation Dialogs   │
└──────────────────────┬──────────────────────────────┘
                       │ Tauri IPC (invoke + events)
┌──────────────────────▼──────────────────────────────┐
│              Tauri v2 Control Plane                   │
│  ┌──────────┐ ┌───────────┐ ┌──────────────────┐   │
│  │ Commands │→│ Services  │→│ ModelRouter      │   │
│  │  (thin)  │ │ (app层)   │ │ (provider select)│   │
│  └──────────┘ └─────┬─────┘ └────────┬─────────┘   │
│                     │                 │              │
│  ┌──────────────────▼─────────────────▼───────────┐ │
│  │              SQLite (sqlx)                       │ │
│  │  projects │ jobs │ assets │ model_profiles │ ... │ │
│  └────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────┐
│                 Data Plane (Workers)                  │
│  ImageGenWorker │ AudioGenWorker │ ... (tokio tasks) │
│  Replicate │ Fal │ HuggingFace │ Kie │ ElevenLabs   │
└─────────────────────────────────────────────────────┘
```

### Rust Crates

| Crate | Purpose |
|-------|---------|
| `artifex-shared-kernel` | Domain IDs, paths, errors, events, time utilities |
| `artifex-asset-management` | Project aggregate, Asset aggregate, repository traits |
| `artifex-job-queue` | Job aggregate, queue lifecycle, repository traits |
| `artifex-model-config` | Provider traits, ModelRouter, ProviderRegistry, credential store |

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Workers resolve providers at job time (not startup) | Enables dynamic provider switching via Settings |
| Credentials stored in OS keychain (not DB) | Secrets never touch disk in plaintext |
| `ModelRouter` with ordered fallback chain | Graceful degradation when providers fail |
| Capability-specific traits (`ImageProvider`, `AudioProvider`, `TtsProvider`) | Clean separation per AI domain |
| Canonical `provider_id` (lowercase) separate from display `name` | Eliminates casing mismatches between frontend and backend |

## AI Providers

| Provider | Capabilities | Auth |
|----------|-------------|------|
| [Replicate](https://replicate.com) | Image generation (FLUX, SDXL) | API key |
| [Fal.ai](https://fal.ai) | Image generation (FLUX Dev) | API key |
| [HuggingFace](https://huggingface.co) | Image generation (SDXL, custom) | API token |
| [Kie.ai](https://kie.ai) | Image generation (Flux Kontext) | API key |
| [ElevenLabs](https://elevenlabs.io) | TTS, SFX, Music | API key |
| [Together AI](https://together.ai) | Text generation (Llama, Mistral) | API key |

Adding a new provider requires implementing the relevant trait (`ImageProvider`, `AudioProvider`, etc.) and registering it in `ProviderRegistry`.

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) 1.80+
- [Node.js](https://nodejs.org/) 20+
- [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

### Install

```bash
# Clone the repository
git clone git@github.com:Rubentxu/artifex.git
cd artifex

# Install frontend dependencies
cd src && npm install && cd ..

# Run in development mode
cd src-tauri && cargo tauri dev
```

### Build for Production

```bash
cd src-tauri && cargo tauri build
```

## Project Structure

```
artifex/
├── crates/                          # Rust workspace members (DDD)
│   ├── artifex-shared-kernel/       # IDs, errors, events, paths
│   ├── artifex-asset-management/    # Project + Asset domains
│   ├── artifex-job-queue/           # Job queue domain
│   └── artifex-model-config/        # Provider abstraction + routing
├── src-tauri/                       # Tauri v2 application
│   ├── src/
│   │   ├── application/             # Application services
│   │   ├── commands/                # Thin IPC adapters
│   │   ├── model_config/            # Provider adapters + service
│   │   │   └── providers/           # Replicate, Fal, HF, Kie, ElevenLabs
│   │   ├── repositories/            # SQLite implementations
│   │   ├── workers/                 # Background job processors
│   │   └── migrations/              # SQL schema migrations
│   └── Cargo.toml
├── src/                             # SvelteKit frontend
│   └── src/
│       ├── lib/
│       │   ├── api/                 # Tauri IPC wrappers
│       │   ├── components/          # Svelte UI components
│       │   ├── stores/              # Svelte stores
│       │   └── types/               # TypeScript types
│       └── routes/                  # SvelteKit routes
├── e2e/                             # Playwright E2E tests
└── investigacion/                   # Architecture docs (Spanish)
```

## Testing

```bash
# Rust tests (unit + integration)
cargo test --workspace

# Frontend tests
cd src && npm test

# E2E tests (requires running app)
cd e2e && npx playwright test
```

**256 tests** — 213 Rust + 43 frontend.

## Configuration

### Setting Up Providers

1. Open Artifex and navigate to **Settings**
2. Enter your API key for each provider you want to use
3. Click **Test Connection** to verify
4. Select your preferred model for each operation type via **Model Selector**

Credentials are stored in your **OS keychain** (macOS Keychain, Linux Secret Service, Windows Credential Manager).

### Custom Prompts

Create reusable prompt templates in **Settings → Prompt Templates** with variable interpolation:

```
A {{style}} pixel art {{subject}} for a {{genre}} game, {{mood}} atmosphere
```

## Roadmap

- [x] Project management (create, delete, list)
- [x] Job queue with async workers
- [x] Asset pipeline with file storage
- [x] Provider abstraction with routing + fallback
- [x] Settings UI with provider management
- [x] Image generation (Replicate, Fal, HuggingFace, Kie)
- [x] Audio/TTS generation (ElevenLabs)
- [x] Keychain credential storage
- [ ] Real-time generation progress (SSE/WebSocket)
- [ ] Generation history with retry
- [ ] Tileset and sprite sheet generation
- [ ] PBR material generation
- [ ] Coding agent with AI
- [ ] Plugin system for custom providers

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop runtime | [Tauri v2](https://v2.tauri.app/) |
| Backend | [Rust](https://www.rust-lang.org/) with DDD architecture |
| Database | [SQLite](https://www.sqlite.org/) via [sqlx](https://github.com/launchbadge/sqlx) |
| Frontend | [SvelteKit](https://kit.svelte.dev/) + [Tailwind CSS](https://tailwindcss.com/) |
| E2E tests | [Playwright](https://playwright.dev/) |
| Credential storage | OS keychain via [keyring](https://crates.io/crates/keyring) |

## License

This project is licensed under the MIT License — see the [LICENSE](./LICENSE) file for details.
