# Arquitectura Técnica: Suite Game Dev con IA usando Rust + Tauri v2

> Investigación para una aplicación tipo "Sorceress" — suite de herramientas para game dev con IA
> Fecha: 2026-04-14

---

## Tabla de Contenidos

1. [Tauri v2 — Estado actual y capacidades](#1-tauri-v2--estado-actual-y-capacidades)
2. [Frontend (UI)](#2-frontend-ui)
3. [Backend Rust](#3-backend-rust)
4. [Arquitectura de Plugins/Módulos](#4-arquitectura-de-pluginsmódulos)
5. [Pipeline de Procesamiento](#5-pipeline-de-procesamiento)
6. [Empaquetado y Distribución](#6-empaquetado-y-distribución)
7. [Patrón Control Plane + Data Plane](#7-patrón-control-plane--data-plane)
8. [Arquitectura Propuesta](#8-arquitectura-propuesta)
9. [Decisiones y Trade-offs](#9-decisiones-y-trade-offs)

---

## 1. Tauri v2 — Estado actual y capacidades

### 1.1 Tauri v2 vs v1: Cambios principales

Tauri v2 (lanzado 2024, estable a Oct 2024) introduce cambios arquitecturales fundamentales:

#### Sistema de permisos (mayor cambio)

En v1 existía un `allowlist` monolítico en `tauri.conf.json`. En v2 se reemplazó por un sistema de **Capabilities + Permissions** granular:

```
src-tauri/capabilities/
  default.json      ← permisos asignados a ventanas
  mobile.json       ← permisos específicos mobile
```

```json
// capabilities/default.json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "fs:allow-read-app-data",
    "shell:allow-execute",
    {
      "identifier": "shell:allow-execute",
      "allow": [{ "name": "binaries/ai-worker", "sidecar": true }]
    }
  ]
}
```

#### Cambios en la API Rust

| v1                              | v2                                          |
|---------------------------------|---------------------------------------------|
| `tauri::Window`                 | `tauri::WebviewWindow`                      |
| `Manager::get_window()`         | `Manager::get_webview_window()`             |
| `api::dialog`, `api::http`      | Extraídos a plugins (`tauri-plugin-dialog`) |
| `App::global_shortcut_manager`  | `tauri-plugin-global-shortcut`              |
| `api::process::Command`         | `tauri-plugin-shell`                        |
| `tauri::updater`                | `tauri-plugin-updater`                      |

#### Cambios en JavaScript API

```typescript
// v1
import { invoke } from '@tauri-apps/api/tauri'
import { readBinaryFile } from '@tauri-apps/api/fs'

// v2
import { invoke } from '@tauri-apps/api/core'
import { readFile } from '@tauri-apps/plugin-fs'
```

#### Soporte Mobile (iOS/Android)

La diferencia estructural más importante: para compilar móvil, el crate debe exponer una **shared library**:

```toml
# src-tauri/Cargo.toml
[lib]
name = "app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]
```

```rust
// src-tauri/src/lib.rs
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// src-tauri/src/main.rs
fn main() {
    app_lib::run();
}
```

#### Multiwebview (experimental)

v2 introduce soporte para múltiples WebViews por ventana (detrás del feature flag `unstable`). Útil para paneles independientes en una suite de herramientas.

### 1.2 Sistema de Plugins Oficial

Plugins disponibles en `@tauri-apps/plugin-*` / `tauri-plugin-*`:

| Plugin              | Rust crate                        | JS package                            | Plataformas           |
|---------------------|-----------------------------------|---------------------------------------|-----------------------|
| File System         | `tauri-plugin-fs`                 | `@tauri-apps/plugin-fs`               | Todas                 |
| HTTP Client         | `tauri-plugin-http`               | `@tauri-apps/plugin-http`             | Todas                 |
| Shell / Sidecar     | `tauri-plugin-shell`              | `@tauri-apps/plugin-shell`            | Desktop               |
| Dialog              | `tauri-plugin-dialog`             | `@tauri-apps/plugin-dialog`           | Todas                 |
| SQL                 | `tauri-plugin-sql`                | `@tauri-apps/plugin-sql`              | Todas                 |
| Store (KV)          | `tauri-plugin-store`              | `@tauri-apps/plugin-store`            | Todas                 |
| Updater             | `tauri-plugin-updater`            | `@tauri-apps/plugin-updater`          | Desktop               |
| Notification        | `tauri-plugin-notification`       | `@tauri-apps/plugin-notification`     | Todas                 |
| Global Shortcut     | `tauri-plugin-global-shortcut`    | `@tauri-apps/plugin-global-shortcut`  | Desktop               |
| OS Info             | `tauri-plugin-os`                 | `@tauri-apps/plugin-os`               | Todas                 |
| Window State        | `tauri-plugin-window-state`       | `@tauri-apps/plugin-window-state`     | Desktop               |
| Localhost           | `tauri-plugin-localhost`          | —                                     | Desktop (producción)  |
| Stronghold          | `tauri-plugin-stronghold`         | `@tauri-apps/plugin-stronghold`       | Todas                 |
| Websocket           | `tauri-plugin-websocket`          | `@tauri-apps/plugin-websocket`        | Todas                 |

**Para una suite de game dev, los más relevantes:**
- `plugin-fs` — acceso a assets del proyecto
- `plugin-dialog` — file pickers para importar sprites/audio
- `plugin-shell` + sidecar — lanzar workers de IA
- `plugin-sql` — base de datos de proyectos
- `plugin-store` — configuración persistente
- `plugin-updater` — actualizaciones automáticas
- `plugin-localhost` — servidor local para previews

### 1.3 IPC: Commands, Events, Channels

#### Commands (call-and-response)

Mecanismo principal para llamar Rust desde el frontend:

```rust
// src-tauri/src/commands/image.rs
#[tauri::command]
async fn generate_sprite(
    prompt: String,
    width: u32,
    height: u32,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<SpriteResult, AppError> {
    let processor = state.image_processor.lock().await;
    processor.generate(prompt, width, height).await
}
```

```typescript
// frontend
import { invoke } from '@tauri-apps/api/core';
const result = await invoke<SpriteResult>('generate_sprite', {
    prompt: 'pixel art knight',
    width: 64,
    height: 64,
});
```

**Nota importante:** Los commands en `lib.rs` NO pueden ser `pub` (limitación del codegen). En módulos separados sí.

#### Events (fire-and-forget, bidireccional)

Para notificaciones asíncronas (progreso de jobs, cambios de estado):

```rust
// Emitir desde Rust hacia frontend
use tauri::Emitter;

app.emit("job-progress", JobProgress { 
    job_id: "abc123",
    percent: 42.0,
    message: "Generando sprites..."
}).unwrap();

// Emitir a una ventana específica
app.emit_to("editor", "sprite-ready", sprite_data).unwrap();
```

```typescript
// Escuchar en frontend
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<JobProgress>('job-progress', (event) => {
    console.log(`Progress: ${event.payload.percent}%`);
});

// Cleanup cuando el componente se desmonta
onUnmount(() => unlisten());
```

#### Channels (streaming de datos, recomendado para IA)

Para streaming de datos grandes (tokens de LLM, chunks de imagen generada):

```rust
use tokio::io::AsyncReadExt;

#[tauri::command]
async fn stream_ai_generation(
    prompt: String,
    on_chunk: tauri::ipc::Channel<GenerationChunk>,
) {
    let mut generator = AIGenerator::new(prompt);
    
    while let Some(chunk) = generator.next_chunk().await {
        on_chunk.send(chunk).unwrap();
    }
}
```

```typescript
import { Channel, invoke } from '@tauri-apps/api/core';

const channel = new Channel<GenerationChunk>();
channel.onmessage = (chunk) => {
    appendToCanvas(chunk);
};

await invoke('stream_ai_generation', { prompt, onChunk: channel });
```

**Channels son preferibles a Events para datos continuos** porque no serializan como JSON sino como binarios.

#### Raw binary IPC (para imágenes/audio)

Para transferir buffers grandes sin overhead de JSON:

```rust
use tauri::ipc::Response;

#[tauri::command]
fn get_sprite_data(sprite_id: String) -> Response {
    let data = load_sprite_bytes(&sprite_id);
    tauri::ipc::Response::new(data)  // no serializa a JSON
}
```

```typescript
const data = await invoke<ArrayBuffer>('get_sprite_data', 
    new Uint8Array([...]),  // raw body
    { headers: { 'X-Sprite-Id': 'hero-idle' } }
);
```

### 1.4 Sidecar: Procesos externos

Para IA pesada (inferencia de modelos grandes) que no queremos en el proceso principal:

```toml
# tauri.conf.json
{
  "bundle": {
    "externalBin": [
      "binaries/ai-worker",
      "binaries/audio-processor"
    ]
  }
}
```

Los binarios deben tener el target triple como sufijo:
- Linux x86_64: `ai-worker-x86_64-unknown-linux-gnu`
- macOS ARM: `ai-worker-aarch64-apple-darwin`
- Windows: `ai-worker-x86_64-pc-windows-msvc.exe`

```rust
// Lanzar sidecar desde Rust
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

#[tauri::command]
async fn start_ai_worker(
    model: String,
    app: tauri::AppHandle,
) -> Result<u32, String> {
    let (mut rx, child) = app.shell()
        .sidecar("ai-worker")
        .unwrap()
        .args(["--model", &model, "--port", "7878"])
        .spawn()
        .map_err(|e| e.to_string())?;
    
    // Capturar stdout/stderr del sidecar
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    let msg = String::from_utf8_lossy(&line);
                    // forward to frontend
                    app.emit("worker-log", msg.to_string()).ok();
                }
                CommandEvent::Terminated(status) => {
                    app.emit("worker-stopped", status.code).ok();
                    break;
                }
                _ => {}
            }
        }
    });
    
    Ok(child.pid())
}
```

### 1.5 WebView por plataforma

| Plataforma | Motor WebView        | Notas                                               |
|------------|----------------------|-----------------------------------------------------|
| Linux      | WebKitGTK (webkit2gtk) | Requiere webkit2gtk 2.40+ para IPC bodies         |
| Windows    | WebView2 (Chromium)  | Requiere WebView2 Runtime (auto-instalable)         |
| macOS      | WKWebView (Safari)   | Nativo del OS, muy eficiente en memoria             |
| iOS        | WKWebView            | Igual que macOS                                     |
| Android    | Android WebView      | Basado en Chromium del sistema                      |

**Implicación:** El CSS/JS debe testearse en WebKit (Safari). Algunas APIs modernas pueden no estar disponibles en versiones antiguas de webkit2gtk en Linux.

**Gestión de ventanas:** TAO (fork de winit) + WRY (abstracción sobre los WebViews).

---

## 2. Frontend (UI)

### 2.1 Framework recomendado: Svelte/SvelteKit o React

#### Comparativa para Tauri v2 game dev suite

| Framework   | Bundle size | Reactividad      | Ecosystem UI | Canvas/WebGL | Veredicto         |
|-------------|-------------|------------------|--------------|--------------|-------------------|
| **Svelte**  | ~20KB       | Signals nativos  | Moderado     | Bueno        | Mejor para apps reactivas sin overhead |
| **React**   | ~130KB      | Virtual DOM      | Excelente    | Excelente    | Más ecosistema, más weight |
| **Solid**   | ~25KB       | Signals reactivos| Moderado     | Bueno        | Muy rápido, ecosystem pequeño |
| Vanilla TS  | ~0KB        | Manual           | N/A          | Excelente    | Solo si dominas la complejidad |

**Recomendación:** **SvelteKit** (con Vite) para la shell/UI principal + React si el equipo ya lo conoce bien.

- Tauri soporta oficialmente SvelteKit con configuración SSG (`adapter-static`)
- El overhead de React es trivial en desktop (no hay red que optimizar)
- Svelte compila a JS vanilla, sin runtime, ideal para Tauri

### 2.2 Librerías UI

#### shadcn/ui (React) — Recomendado si se usa React

```bash
npx shadcn@latest init
```

- Componentes copiables (no dependencia, son tuyos)
- Tailwind CSS + Radix UI bajo el capó
- Excelente accesibilidad, theme system, dark mode
- Compatible con Tauri (no usa Node APIs)

#### Melt UI (Svelte) — Equivalente para Svelte

```bash
npm install @melt-ui/svelte
```

#### TailwindCSS

Imprescindible para cualquier opción:

```typescript
// tailwind.config.ts
export default {
    content: ['./src/**/*.{html,js,svelte,ts}'],
    theme: {
        extend: {
            colors: {
                // paleta game dev dark theme
                canvas: '#1a1a2e',
                panel: '#16213e',
                accent: '#e94560',
            }
        }
    }
}
```

### 2.3 Canvas/WebGL para editor de sprites

#### PixiJS — Recomendado para editor de sprites 2D

```bash
npm install pixi.js
```

- WebGL/WebGPU renderer con fallback Canvas2D
- Excelente rendimiento para sprites, tilemaps
- API familiar para game dev
- Soporte para spritesheets, animaciones

```typescript
import * as PIXI from 'pixi.js';

const app = new PIXI.Application();
await app.init({
    width: 512, height: 512,
    backgroundColor: 0x1a1a2e,
    antialias: false,  // pixel art
    resolution: window.devicePixelRatio,
});

document.getElementById('canvas-container')!.appendChild(app.canvas);

// Cargar sprite desde Tauri
const spriteData = await invoke<Uint8Array>('get_sprite_data', { id });
const texture = PIXI.Texture.from(new Uint8Array(spriteData));
const sprite = new PIXI.Sprite(texture);
app.stage.addChild(sprite);
```

#### Konva.js — Para edición con transformaciones (resize, rotate, etc.)

```bash
npm install konva react-konva  # o svelte-konva
```

- Mejor para editors con selección, drag, layers
- Canvas2D (no WebGL), más compatible con WebKit

#### Fabric.js — Para edición de objetos vectoriales

```bash
npm install fabric
```

- Canvas2D con objetos manipulables
- Bueno para composición de assets

#### Comparativa Canvas libs

| Librería   | Renderer      | Casos de uso                          | Performance |
|------------|---------------|---------------------------------------|-------------|
| **PixiJS** | WebGL/WebGPU  | Sprite viewer, animación, tilemaps    | Excelente   |
| **Konva**  | Canvas2D      | Editor interactivo, layers, drag      | Bueno       |
| **Fabric** | Canvas2D      | Composición de assets, vector editing | Bueno       |
| Three.js   | WebGL         | Preview 3D de assets                  | Excelente   |

### 2.4 Estado: Zustand / Jotai / Signals

#### Para React: Zustand (recomendado)

```bash
npm install zustand
```

```typescript
import { create } from 'zustand';

interface ProjectStore {
    currentTool: string;
    sprites: Sprite[];
    setTool: (tool: string) => void;
    addSprite: (sprite: Sprite) => void;
}

const useProjectStore = create<ProjectStore>((set) => ({
    currentTool: 'brush',
    sprites: [],
    setTool: (tool) => set({ currentTool: tool }),
    addSprite: (sprite) => set((s) => ({ sprites: [...s.sprites, sprite] })),
}));
```

#### Para React: Jotai (atómico, más granular)

```bash
npm install jotai
```

```typescript
import { atom, useAtom } from 'jotai';

const currentToolAtom = atom('brush');
const spritesAtom = atom<Sprite[]>([]);

// En componente
const [tool, setTool] = useAtom(currentToolAtom);
```

#### Para Svelte: Stores nativos + @preact/signals-react

Svelte tiene reactividad built-in. Para estado global:

```typescript
// stores/project.ts
import { writable, derived } from 'svelte/store';

export const sprites = writable<Sprite[]>([]);
export const currentTool = writable('brush');
export const selectedSprite = derived(sprites, $sprites => 
    $sprites.find(s => s.selected)
);
```

---

## 3. Backend Rust

### 3.1 Organización: Workspace con Crates

```
sorceress/
├── Cargo.toml               ← workspace root
├── src-tauri/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       └── commands/
├── crates/
│   ├── sorceress-core/      ← tipos compartidos, traits
│   ├── sorceress-image/     ← procesamiento de imágenes
│   ├── sorceress-audio/     ← procesamiento de audio
│   ├── sorceress-ai/        ← integración modelos IA
│   ├── sorceress-gpu/       ← compute GPU con wgpu
│   └── sorceress-db/        ← capa de datos (SQLite)
└── workers/
    └── ai-worker/           ← sidecar: proceso de inferencia IA
```

```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "src-tauri",
    "crates/sorceress-core",
    "crates/sorceress-image",
    "crates/sorceress-audio",
    "crates/sorceress-ai",
    "crates/sorceress-gpu",
    "crates/sorceress-db",
    "workers/ai-worker",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### 3.2 Crates esenciales

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-shell = "2"
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
tauri-plugin-store = "2"
tauri-plugin-updater = "2"
tauri-plugin-http = "2"
tauri-plugin-notification = "2"

# Async runtime
tokio = { workspace = true }

# Serialización
serde = { workspace = true }
serde_json = { workspace = true }

# HTTP server embebido (para API local del sidecar)
axum = "0.8"

# Base de datos
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "migrate"] }

# Observabilidad
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# File watching
notify = "7"
notify-debouncer-mini = "0.4"

# Crypto / hashing
sha2 = "0.10"
```

### 3.3 Procesamiento de Imágenes

#### `image` crate (recomendado, pure Rust)

```toml
image = { version = "0.25", features = ["png", "jpeg", "webp", "gif"] }
imageproc = "0.25"  # operaciones avanzadas
```

```rust
use image::{DynamicImage, ImageFormat, GenericImageView};

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn resize_sprite(
        data: &[u8],
        width: u32,
        height: u32,
    ) -> anyhow::Result<Vec<u8>> {
        let img = image::load_from_memory(data)?;
        let resized = img.resize_exact(width, height, image::imageops::FilterType::Nearest);
        
        let mut output = Vec::new();
        resized.write_to(&mut std::io::Cursor::new(&mut output), ImageFormat::Png)?;
        Ok(output)
    }
    
    pub fn extract_palette(data: &[u8], max_colors: usize) -> anyhow::Result<Vec<[u8; 4]>> {
        let img = image::load_from_memory(data)?.to_rgba8();
        // extraer colores únicos...
        Ok(vec![])
    }
}
```

#### `opencv-rust` (para CV avanzado)

Solo necesario si se requiere detección de objetos, background removal, etc. Tiene dependencias nativas C++:

```toml
opencv = { version = "0.93", features = ["imgproc", "imgcodecs"] }
```

**Advertencia:** opencv-rust aumenta significativamente el tamaño del binario y la complejidad del build. Considerar solo si candle no es suficiente.

### 3.4 Procesamiento de Audio

#### `rodio` + `symphonia` (recomendado)

```toml
rodio = "0.20"      # playback
symphonia = { version = "0.5", features = ["all"] }  # decode/encode
```

```rust
use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;

pub struct AudioPlayer {
    sink: Option<Sink>,
    _stream: OutputStream,
}

impl AudioPlayer {
    pub fn play_file(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let file = std::fs::File::open(path)?;
        let source = Decoder::new(BufReader::new(file))?;
        
        if let Some(sink) = &self.sink {
            sink.append(source);
        }
        Ok(())
    }
}
```

#### `hound` para WAV processing

```toml
hound = "3.5"
```

### 3.5 GPU Compute: `wgpu`

Para efectos de imagen, shaders, procesamiento paralelo en GPU:

```toml
wgpu = { version = "22", features = ["vulkan", "metal", "dx12"] }
bytemuck = { version = "1", features = ["derive"] }
```

```rust
use wgpu::*;

pub struct GpuProcessor {
    device: Device,
    queue: Queue,
}

impl GpuProcessor {
    pub async fn new() -> anyhow::Result<Self> {
        let instance = Instance::default();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .ok_or(anyhow::anyhow!("No GPU adapter found"))?;
        
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await?;
        
        Ok(Self { device, queue })
    }
    
    pub async fn apply_shader(
        &self, 
        image_data: &[u8],
        shader_wgsl: &str,
    ) -> anyhow::Result<Vec<u8>> {
        // Compilar shader y ejecutar compute pass
        todo!()
    }
}
```

### 3.6 Integración con Modelos IA Locales

#### `candle` (HuggingFace) — Recomendado principal

```toml
candle-core = "0.9"
candle-nn = "0.9"
candle-transformers = "0.9"
hf-hub = { version = "0.3", features = ["tokio"] }
tokenizers = "0.21"
```

**Capacidades de candle:**
- Soporte GPU: CUDA, Metal (macOS), CPU optimizado (MKL/Accelerate)
- Modelos soportados: LLaMA, Mistral, Phi, Stable Diffusion, Whisper, BERT
- Formato de pesos: safetensors, GGUF (quantized), npz
- API similar a PyTorch

```rust
use candle_core::{Device, Tensor, DType};
use candle_transformers::models::stable_diffusion;

pub struct ImageGenerator {
    model: stable_diffusion::StableDiffusion,
    device: Device,
}

impl ImageGenerator {
    pub async fn generate(
        &self,
        prompt: &str,
        width: usize,
        height: usize,
        steps: usize,
    ) -> anyhow::Result<Vec<u8>> {
        // Inferencia en GPU/CPU
        let latents = self.model.run(prompt, width, height, steps)?;
        let image = latents.to_rgb8()?;
        Ok(image)
    }
}
```

#### `tract` (ONNX/TensorFlow) — Para modelos ONNX

```toml
tract-onnx = "0.21"
```

```rust
use tract_onnx::prelude::*;

let model = tract_onnx::onnx()
    .model_for_path("model.onnx")?
    .into_optimized()?
    .into_runnable()?;
```

**Casos de uso:** Modelos ONNX exportados desde PyTorch/TensorFlow, más portables.

#### `burn` — Framework ML alternativo

```toml
burn = { version = "0.16", features = ["wgpu"] }
```

**Casos de uso:** Si se quiere entrenamiento + inferencia, usa GPU a través de wgpu (sin dependencia CUDA directa).

#### Comparativa IA local

| Framework      | GPU         | ONNX  | Quantized | Modelos pre-incluidos | Madurez |
|----------------|-------------|-------|-----------|----------------------|---------|
| **candle**     | CUDA/Metal  | Sí    | GGUF      | Muchos               | Alta    |
| **tract**      | CPU only    | Sí    | Limitado  | Ninguno              | Alta    |
| **burn**       | wgpu/CUDA   | No    | No        | Ninguno              | Media   |
| **llama.cpp** (via FFI) | CUDA/Metal | No | GGUF | Vía CLI | Muy alta |

**Recomendación:** candle para inferencia interna en Rust, considerar llama.cpp como sidecar para modelos LLM grandes (mejor optimización GGUF).

### 3.7 HTTP Server Embebido: Axum

Para exponer una API local que el frontend puede consumir (además de IPC de Tauri):

```toml
axum = "0.8"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
```

```rust
use axum::{Router, Json, extract::State};
use tower_http::cors::CorsLayer;
use std::sync::Arc;

pub async fn start_local_server(state: Arc<AppState>, port: u16) {
    let app = Router::new()
        .route("/api/generate", axum::routing::post(generate_handler))
        .route("/api/models", axum::routing::get(list_models))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind(
        format!("127.0.0.1:{}", port)
    ).await.unwrap();
    
    axum::serve(listener, app).await.unwrap();
}

async fn generate_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Json<GenerateResponse> {
    // ...
}
```

**Nota:** El plugin `tauri-plugin-localhost` permite servir el frontend desde un servidor local embebido en producción (útil para algunos casos).

### 3.8 Base de Datos Local: SQLx + SQLite

```toml
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "migrate", "json"] }
```

```
src-tauri/
  migrations/
    0001_initial.sql
    0002_add_sprites.sql
```

```sql
-- migrations/0001_initial.sql
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    settings JSON
);

CREATE TABLE sprites (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    name TEXT NOT NULL,
    data BLOB,
    metadata JSON
);
```

```rust
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

pub async fn create_pool(db_path: &str) -> anyhow::Result<SqlitePool> {
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);
    
    let pool = SqlitePool::connect_with(opts).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
```

Alternativa: `tauri-plugin-sql` que envuelve sqlx y expone la DB al frontend JS directamente.

### 3.9 File Watching: `notify`

Para detectar cambios en assets del proyecto (hot-reload de sprites, etc.):

```toml
notify = "7"
notify-debouncer-mini = "0.4"
```

```rust
use notify::{Watcher, RecursiveMode, Event, EventKind};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};

pub fn watch_project_folder(
    path: &std::path::Path,
    app: tauri::AppHandle,
) -> anyhow::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    
    let mut debouncer = new_debouncer(
        std::time::Duration::from_millis(500),
        tx,
    )?;
    
    debouncer.watcher().watch(path, RecursiveMode::Recursive)?;
    
    std::thread::spawn(move || {
        for events in rx.flatten() {
            for event in events {
                app.emit("file-changed", FileChangeEvent {
                    path: event.path.to_string_lossy().to_string(),
                    kind: format!("{:?}", event.kind),
                }).ok();
            }
        }
    });
    
    Ok(())
}
```

---

## 4. Arquitectura de Plugins/Módulos

### 4.1 Sistema de Plugin con Trait Rust

Cada herramienta de la suite (generador de sprites, generador de audio, editor de tiles, etc.) se modela como un plugin:

```rust
// crates/sorceress-core/src/plugin.rs
use async_trait::async_trait;
use serde_json::Value;
use tauri::AppHandle;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,  // "image-gen", "audio-gen", "sprite-edit"
    pub ui_entry: Option<String>,   // ruta al componente frontend
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn manifest(&self) -> &PluginManifest;
    
    async fn initialize(&self, app: &AppHandle) -> anyhow::Result<()>;
    
    async fn execute(
        &self,
        command: &str,
        params: Value,
        app: &AppHandle,
    ) -> anyhow::Result<Value>;
    
    async fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// Plugin para generación de sprites con IA
pub struct SpriteGeneratorPlugin {
    manifest: PluginManifest,
    generator: tokio::sync::Mutex<Option<ImageGenerator>>,
}

#[async_trait]
impl Plugin for SpriteGeneratorPlugin {
    fn manifest(&self) -> &PluginManifest { &self.manifest }
    
    async fn initialize(&self, _app: &AppHandle) -> anyhow::Result<()> {
        let mut gen = self.generator.lock().await;
        *gen = Some(ImageGenerator::load_model("stable-diffusion-xl").await?);
        Ok(())
    }
    
    async fn execute(
        &self,
        command: &str,
        params: Value,
        app: &AppHandle,
    ) -> anyhow::Result<Value> {
        match command {
            "generate" => {
                let prompt = params["prompt"].as_str().unwrap_or("");
                // ... generar sprite
                Ok(serde_json::json!({ "status": "ok", "sprite_id": "abc" }))
            }
            _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
        }
    }
}
```

### 4.2 Plugin Registry Dinámico

```rust
// crates/sorceress-core/src/registry.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: RwLock::new(HashMap::new()) }
    }
    
    pub async fn register(&self, plugin: Arc<dyn Plugin>) {
        let id = plugin.manifest().id.clone();
        self.plugins.write().await.insert(id, plugin);
    }
    
    pub async fn get(&self, id: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.read().await.get(id).cloned()
    }
    
    pub async fn list(&self) -> Vec<PluginManifest> {
        self.plugins.read().await
            .values()
            .map(|p| p.manifest().clone())
            .collect()
    }
    
    pub async fn execute(
        &self,
        plugin_id: &str,
        command: &str,
        params: serde_json::Value,
        app: &tauri::AppHandle,
    ) -> anyhow::Result<serde_json::Value> {
        let plugin = self.get(plugin_id).await
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", plugin_id))?;
        
        plugin.execute(command, params, app).await
    }
}
```

### 4.3 Comunicación Plugin → Frontend via Events

```rust
// Patrón para progreso y resultados de plugins
#[derive(serde::Serialize, Clone)]
pub struct PluginEvent {
    pub plugin_id: String,
    pub event_type: String,  // "progress", "result", "error"
    pub payload: serde_json::Value,
}

// En un plugin
impl SpriteGeneratorPlugin {
    async fn generate_with_progress(
        &self,
        params: &GenerateParams,
        app: &tauri::AppHandle,
    ) -> anyhow::Result<String> {
        use tauri::Emitter;
        
        // Emitir progreso
        app.emit("plugin-event", PluginEvent {
            plugin_id: self.manifest.id.clone(),
            event_type: "progress".to_string(),
            payload: serde_json::json!({ "step": 1, "total": 20, "desc": "Cargando modelo" }),
        })?;
        
        // ... procesamiento
        
        app.emit("plugin-event", PluginEvent {
            plugin_id: self.manifest.id.clone(),
            event_type: "result".to_string(),
            payload: serde_json::json!({ "sprite_id": "new-sprite-id" }),
        })?;
        
        Ok("new-sprite-id".to_string())
    }
}
```

---

## 5. Pipeline de Procesamiento

### 5.1 Cola de Jobs Asíncrona

```rust
// crates/sorceress-core/src/job_queue.rs
use tokio::sync::{mpsc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum JobStatus {
    Queued,
    Running { progress: f32, message: String },
    Completed { result: serde_json::Value },
    Failed { error: String },
    Cancelled,
}

#[derive(Debug)]
pub struct Job {
    pub id: String,
    pub plugin_id: String,
    pub command: String,
    pub params: serde_json::Value,
    pub cancel_token: tokio_util::sync::CancellationToken,
}

pub struct JobQueue {
    tx: mpsc::Sender<Job>,
    statuses: Arc<Mutex<HashMap<String, JobStatus>>>,
}

impl JobQueue {
    pub fn new(
        registry: Arc<PluginRegistry>,
        app: tauri::AppHandle,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel::<Job>(100);
        let statuses = Arc::new(Mutex::new(HashMap::new()));
        let statuses_clone = statuses.clone();
        
        tokio::spawn(async move {
            // Procesar jobs concurrentemente (con límite)
            let semaphore = Arc::new(tokio::sync::Semaphore::new(4)); // max 4 jobs paralelos
            
            while let Some(job) = rx.recv().await {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let registry = registry.clone();
                let app = app.clone();
                let statuses = statuses_clone.clone();
                
                tokio::spawn(async move {
                    let _permit = permit; // se libera al caer este scope
                    
                    // Actualizar estado a Running
                    statuses.lock().await.insert(
                        job.id.clone(),
                        JobStatus::Running { progress: 0.0, message: "Iniciando...".to_string() },
                    );
                    app.emit("job-status", (&job.id, &JobStatus::Running { 
                        progress: 0.0, 
                        message: "Iniciando...".to_string() 
                    })).ok();
                    
                    // Ejecutar el job con soporte de cancelación
                    let result = tokio::select! {
                        r = registry.execute(&job.plugin_id, &job.command, job.params.clone(), &app) => r,
                        _ = job.cancel_token.cancelled() => {
                            Err(anyhow::anyhow!("Job cancelled"))
                        }
                    };
                    
                    // Actualizar estado final
                    let final_status = match result {
                        Ok(value) => JobStatus::Completed { result: value },
                        Err(e) if e.to_string().contains("cancelled") => JobStatus::Cancelled,
                        Err(e) => JobStatus::Failed { error: e.to_string() },
                    };
                    
                    statuses.lock().await.insert(job.id.clone(), final_status.clone());
                    app.emit("job-status", (&job.id, &final_status)).ok();
                });
            }
        });
        
        Self { tx, statuses }
    }
    
    pub async fn submit(&self, plugin_id: String, command: String, params: serde_json::Value) -> String {
        let job_id = Uuid::new_v4().to_string();
        let cancel_token = tokio_util::sync::CancellationToken::new();
        
        self.statuses.lock().await.insert(job_id.clone(), JobStatus::Queued);
        
        self.tx.send(Job {
            id: job_id.clone(),
            plugin_id,
            command,
            params,
            cancel_token,
        }).await.unwrap();
        
        job_id
    }
    
    pub async fn cancel(&self, job_id: &str) -> bool {
        // Buscar y cancelar el token del job
        // (requiere guardar los tokens en un HashMap separado)
        false
    }
}
```

### 5.2 Comandos Tauri para el Pipeline

```rust
// src-tauri/src/commands/jobs.rs
use crate::state::AppState;

#[tauri::command]
pub async fn submit_job(
    plugin_id: String,
    command: String,
    params: serde_json::Value,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    state.job_queue
        .submit(plugin_id, command, params)
        .await
        .pipe(Ok)
}

#[tauri::command]
pub async fn cancel_job(
    job_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    Ok(state.job_queue.cancel(&job_id).await)
}

#[tauri::command]
pub async fn get_job_status(
    job_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<JobStatus, String> {
    state.job_queue
        .status(&job_id)
        .await
        .ok_or("Job not found".to_string())
}
```

---

## 6. Empaquetado y Distribución

### 6.1 Formatos por plataforma

| Plataforma | Formato        | Herramienta          |
|------------|----------------|----------------------|
| Windows    | `.msi`, `.exe` (NSIS) | WiX Toolset, NSIS |
| macOS      | `.dmg`, `.app` | macOS bundler nativo |
| Linux      | `.deb`, `.rpm`, `.AppImage` | cargo-tauri bundler |
| iOS        | `.ipa`         | Xcode               |
| Android    | `.apk`, `.aab` | Android Studio/Gradle |

```toml
# tauri.conf.json (sección bundle)
{
  "productName": "Sorceress",
  "version": "0.1.0",
  "identifier": "com.sorceress.app",
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"],
    "linux": {
      "deb": { "depends": ["webkit2gtk-4.1"] },
      "appimage": { "bundleMediaFramework": true }
    },
    "windows": {
      "webviewInstallMode": { "type": "downloadBootstrapper" }
    },
    "macOS": {
      "minimumSystemVersion": "10.15"
    }
  }
}
```

### 6.2 Tamaño de Binario

Técnicas para reducir tamaño:

```toml
# src-tauri/Cargo.toml - profile release optimizado
[profile.release]
opt-level = "z"       # optimizar para tamaño
lto = true            # Link Time Optimization
codegen-units = 1     # mejor optimización (más lento de compilar)
panic = "abort"       # eliminar unwinding code
strip = true          # strip símbolos de debug
```

**Tamaños típicos:**
- App básica Tauri v2: ~2-5 MB
- Con candle (CPU only): ~20-40 MB
- Con modelos IA embebidos (GGUF 4-bit): ~2-8 GB adicionales (se descargan on-demand)

**Estrategia:** Distribuir sin modelos, descargar en primer uso desde HuggingFace Hub.

### 6.3 Auto-updater

```toml
# tauri.conf.json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://releases.sorceress.app/update/{{target}}/{{arch}}/{{current_version}}"
      ],
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6...",
      "windows": { "installMode": "passive" }
    }
  },
  "bundle": {
    "createUpdaterArtifacts": true
  }
}
```

```typescript
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

async function checkForUpdates() {
    const update = await check();
    if (update?.available) {
        console.log(`Update disponible: ${update.version}`);
        await update.downloadAndInstall();
        await relaunch();
    }
}
```

### 6.4 Code Signing

- **macOS:** Requiere Developer ID certificate de Apple. `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD` en CI.
- **Windows:** Requiere certificado EV de entidad CA. Variable `TAURI_SIGNING_PRIVATE_KEY`.
- **Linux:** GPG signing para repos .deb/.rpm.

GitHub Actions workflow:

```yaml
# .github/workflows/release.yml
- name: Build Tauri App
  uses: tauri-apps/tauri-action@v0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
    APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
    APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
    APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
    APPLE_ID: ${{ secrets.APPLE_ID }}
    APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
    APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
  with:
    tagName: v__VERSION__
    releaseName: "Sorceress v__VERSION__"
    releaseBody: "See release notes"
    releaseDraft: true
    prerelease: false
```

---

## 7. Patrón Control Plane + Data Plane

### 7.1 Concepto

Para una suite de herramientas con procesamiento pesado de IA, se recomienda separar:

```
┌─────────────────────────────────────────────────────────────┐
│                    CONTROL PLANE                            │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              Tauri App (Proceso Principal)            │  │
│  │                                                       │  │
│  │  Frontend (WebView)   │   Rust Backend (tauri)        │  │
│  │  ─────────────────    │   ─────────────────────────   │  │
│  │  • UI / Interacción   │   • Gestión de ventanas       │  │
│  │  • State management   │   • Plugin registry           │  │
│  │  • Canvas/editor      │   • Job queue                 │  │
│  │  • Configuración      │   • SQLite (metadatos)        │  │
│  │                       │   • File watching             │  │
│  └───────────────────────────────────────────────────────┘  │
│                          │ IPC (commands/events/channels)   │
└──────────────────────────┼──────────────────────────────────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
┌─────────────────┐ ┌────────────┐ ┌────────────────┐
│   DATA PLANE    │ │ DATA PLANE │ │   DATA PLANE   │
│                 │ │            │ │                │
│  AI Worker      │ │  GPU Proc  │ │  Audio Worker  │
│  (sidecar)      │ │  (wgpu)    │ │  (sidecar)     │
│                 │ │            │ │                │
│  • Inferencia   │ │  • Shaders │ │  • Synth audio │
│    candle/GGUF  │ │  • Compute │ │  • Analysis    │
│  • REST API     │ │  • Effects │ │  • TTS/STT     │
│    :7878        │ │            │ │    :7879       │
└─────────────────┘ └────────────┘ └────────────────┘
```

### 7.2 Comunicación entre Planes

#### Opción A: Sidecar con REST API (recomendado para IA pesada)

```rust
// workers/ai-worker/src/main.rs
use axum::{Router, Json, extract::State};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(serde::Deserialize)]
struct GenerateRequest {
    prompt: String,
    model: String,
    width: u32,
    height: u32,
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args()
        .find(|a| a.starts_with("--port="))
        .and_then(|a| a.split('=').nth(1)?.parse().ok())
        .unwrap_or(7878);
    
    let state = Arc::new(AIWorkerState::new().await);
    
    let app = Router::new()
        .route("/generate", axum::routing::post(generate))
        .route("/health", axum::routing::get(|| async { "ok" }))
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind(
        format!("127.0.0.1:{}", port)
    ).await.unwrap();
    
    println!("AI Worker listening on port {}", port);  // Tauri captura stdout
    axum::serve(listener, app).await.unwrap();
}
```

```rust
// En el proceso principal Tauri
// Llamar al worker via HTTP local
use tauri_plugin_http::reqwest;

#[tauri::command]
async fn generate_sprite_via_worker(
    prompt: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:7878/generate")
        .json(&serde_json::json!({
            "prompt": prompt,
            "model": "sdxl",
            "width": 512,
            "height": 512,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}
```

#### Opción B: Canales stdout/stdin con el sidecar

Para procesos simples o cuando REST no es necesario:

```rust
// Comunicación via stdin/stdout con protocolo JSON-lines
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

let (mut rx, mut child) = app.shell()
    .sidecar("ai-worker")
    .unwrap()
    .spawn()
    .unwrap();

// Enviar comando
let cmd = serde_json::json!({ "type": "generate", "prompt": "knight" });
child.write(format!("{}\n", cmd).as_bytes()).unwrap();

// Recibir resultado
while let Some(event) = rx.recv().await {
    match event {
        CommandEvent::Stdout(line) => {
            if let Ok(result) = serde_json::from_slice::<serde_json::Value>(&line) {
                // procesar resultado
            }
        }
        _ => {}
    }
}
```

#### Opción C: Todo en proceso (candle embebido en Tauri)

Para modelos pequeños (<2GB) que caben en memoria del proceso principal:

```rust
// src-tauri/src/state.rs
pub struct AppState {
    pub image_generator: Arc<tokio::sync::Mutex<Option<ImageGenerator>>>,
    pub db: SqlitePool,
    pub job_queue: Arc<JobQueue>,
}

impl AppState {
    pub async fn new(app_dir: &std::path::Path) -> Self {
        Self {
            image_generator: Arc::new(tokio::sync::Mutex::new(None)),
            db: create_pool(&app_dir.join("sorceress.db").to_str().unwrap()).await.unwrap(),
            job_queue: Arc::new(JobQueue::new()),
        }
    }
}
```

### 7.3 Recomendación por caso de uso

| Modelo / Tarea                  | Estrategia         | Razón                              |
|---------------------------------|--------------------|------------------------------------|
| LLM grande (>7B params)         | Sidecar + REST     | Memoria aislada, reiniciable        |
| Stable Diffusion XL             | Sidecar + REST     | 4-8 GB VRAM, proceso dedicado       |
| Modelos pequeños (Phi-2, etc.)  | Candle en proceso  | Sin overhead de IPC                 |
| OCR/imagen ligera               | Candle en proceso  | Rápido, sincrónico aceptable        |
| Audio (rodio/symphonia)         | En proceso         | Bajo uso de recursos                |
| GPU compute (shaders)           | wgpu en proceso    | Comparte contexto con WebView       |

---

## 8. Arquitectura Propuesta

### 8.1 Estructura completa del proyecto

```
sorceress/
├── Cargo.toml                   # workspace
├── package.json                 # frontend tooling
├── vite.config.ts
├── tailwind.config.ts
│
├── src/                         # Frontend (SvelteKit/React)
│   ├── app.html
│   ├── routes/
│   │   ├── +layout.svelte       # Shell principal con sidebar
│   │   ├── +page.svelte         # Dashboard de proyectos
│   │   ├── sprite-gen/          # Herramienta: generador de sprites
│   │   ├── sprite-edit/         # Herramienta: editor de sprites
│   │   ├── audio-gen/           # Herramienta: generador de audio
│   │   ├── tile-map/            # Herramienta: editor de tilemaps
│   │   └── settings/            # Configuración
│   ├── lib/
│   │   ├── components/          # Componentes reutilizables
│   │   ├── stores/              # Estado global (Svelte stores/Zustand)
│   │   ├── tauri/               # Wrappers de APIs Tauri
│   │   └── canvas/              # Componentes PixiJS/Konva
│
├── src-tauri/                   # Tauri app
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── migrations/              # SQLite migrations
│   └── src/
│       ├── main.rs
│       ├── lib.rs               # Entry point
│       ├── state.rs             # AppState global
│       ├── commands/            # Tauri commands
│       │   ├── mod.rs
│       │   ├── projects.rs
│       │   ├── sprites.rs
│       │   ├── audio.rs
│       │   └── jobs.rs
│       └── setup.rs             # Inicialización de plugins/estado
│
├── crates/
│   ├── sorceress-core/          # Tipos compartidos, traits de plugin
│   ├── sorceress-image/         # Procesamiento de imágenes
│   ├── sorceress-audio/         # Procesamiento de audio
│   ├── sorceress-ai/            # Integración candle/tract
│   └── sorceress-db/            # Capa de datos
│
└── workers/
    ├── ai-worker/               # Sidecar IA pesada
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs          # axum server
    │       ├── models.rs        # gestión de modelos
    │       └── handlers.rs      # REST handlers
    └── audio-worker/            # Sidecar síntesis audio (opcional)
```

### 8.2 Setup inicial de la aplicación

```rust
// src-tauri/src/lib.rs
mod commands;
mod state;
mod setup;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Plugins oficiales
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        // Logging
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build()
        )
        // Setup estado global
        .setup(|app| {
            setup::initialize(app)?;
            Ok(())
        })
        // Commands
        .invoke_handler(tauri::generate_handler![
            commands::projects::list_projects,
            commands::projects::create_project,
            commands::sprites::generate_sprite,
            commands::sprites::list_sprites,
            commands::audio::generate_audio,
            commands::jobs::submit_job,
            commands::jobs::cancel_job,
            commands::jobs::get_job_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

## 9. Decisiones y Trade-offs

### 9.1 Resumen de elecciones técnicas

| Decisión                    | Elección                    | Alternativa descartada        | Razón                                       |
|-----------------------------|-----------------------------|-------------------------------|---------------------------------------------|
| Framework desktop           | Tauri v2                    | Electron, Flutter             | Tamaño binario, Rust nativo, seguridad      |
| Frontend framework          | SvelteKit o React           | Vanilla TS                    | Productividad, ecosistema                   |
| UI components               | shadcn/ui (React) / Melt UI | Chakra, MUI                   | Customizable, sin vendor lock-in            |
| Canvas editor               | PixiJS + Konva              | Three.js solo                 | 2D optimizado para sprites                  |
| Estado frontend             | Zustand / Svelte stores     | Redux                         | Simplicidad, menor boilerplate              |
| IA inferencia local         | candle (HuggingFace)        | llama.cpp directo, tch-rs     | Pure Rust, sin bindings C++, GPU support    |
| LLM grandes                 | llama.cpp sidecar           | candle en proceso             | Mejor optimización GGUF/quantized           |
| GPU compute shaders         | wgpu                        | OpenGL, vulkano               | Cross-platform, WebGPU compatible           |
| Imagen processing           | image + imageproc           | opencv-rust                   | Pure Rust, sin deps nativas pesadas         |
| Base de datos               | SQLx + SQLite               | sled, rocksdb                 | SQL familiar, migraciones, JSON support     |
| HTTP server embebido        | axum                        | actix-web                     | Ergonómico, tokio nativo, menor overhead    |
| File watching               | notify + debouncer          | inotify directo               | Cross-platform                              |
| Arquitectura sidecar        | Control Plane + Data Plane  | Todo en proceso               | Aislamiento, reiniciabilidad, memoria       |

### 9.2 Riesgos identificados

1. **WebKitGTK en Linux:** Algunas APIs CSS/JS modernas pueden no estar disponibles. Probar en Ubuntu LTS con webkit2gtk-4.1.

2. **candle + GPU en Windows:** Requiere CUDA Toolkit instalado. Considerar distribución de versión CPU-only y descarga opcional CUDA.

3. **Tamaño de modelos IA:** Los modelos SD XL (6GB+) no se pueden bundlear. Implementar descarga on-demand con progreso.

4. **WebView2 en Windows:** El usuario debe tener WebView2 Runtime. Usar `"type": "downloadBootstrapper"` en `webviewInstallMode` para instalación automática.

5. **Sidecar lifecycle:** Si el sidecar se cae, el proceso principal debe detectarlo y reiniciarlo. Implementar health check periódico.

6. **Memory leaks en canvas:** PixiJS mantiene texturas en GPU. Liberar texturas cuando se cierra una herramienta.

### 9.3 Secuencia de implementación recomendada

```
Fase 1 (MVP):
  ✓ Setup Tauri v2 con SvelteKit/React + Vite
  ✓ Sistema de plugins básico (trait + registry)
  ✓ SQLite con sqlx (proyectos, sprites metadata)
  ✓ File system (import/export assets)
  ✓ Canvas viewer básico con PixiJS
  ✓ Job queue con tokio channels

Fase 2 (IA básica):
  ✓ candle integrado (modelos pequeños: Phi-2, SDXL Turbo)
  ✓ Descarga de modelos desde HuggingFace Hub
  ✓ Progreso via Channels
  ✓ Generador de sprites básico

Fase 3 (Sidecar + modelos grandes):
  ✓ AI Worker sidecar con axum
  ✓ Stable Diffusion XL completo
  ✓ Modelo TTS (audio para juegos)
  ✓ File watcher para hot-reload

Fase 4 (Distribución):
  ✓ Auto-updater
  ✓ Code signing (macOS/Windows)
  ✓ CI/CD con GitHub Actions
  ✓ AppImage/DMG/MSI
```

---

## Referencias

- [Tauri v2 Docs](https://v2.tauri.app)
- [Tauri Migration v1→v2](https://v2.tauri.app/start/migrate/from-tauri-1/)
- [candle (HuggingFace)](https://github.com/huggingface/candle)
- [wgpu](https://wgpu.rs)
- [PixiJS](https://pixijs.com)
- [axum](https://github.com/tokio-rs/axum)
- [sqlx](https://github.com/launchbadge/sqlx)
- [shadcn/ui](https://ui.shadcn.com)
- [tauri-plugin-sql](https://github.com/tauri-apps/tauri-plugin-sql)
- [notify (file watching)](https://github.com/notify-rs/notify)
