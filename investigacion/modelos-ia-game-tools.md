# Modelos IA para App de Creación de Juegos — Abril 2026

**Propósito:** Referencia técnica para seleccionar modelos IA, APIs y librerías Rust para
una suite de herramientas de game development (proyecto Sorceress / NotebookAI).
Cubre tanto modelos open source (OSS / self-hosted) como APIs de pago frontier.

> **Nota de actualización:** Este documento integra la investigación original de abril 2025
> con los modelos frontier publicados o actualizados significativamente entre abril 2025
> y abril 2026. Para cada categoría se presentan primero las APIs de pago (estado del arte)
> y después las opciones OSS / self-hosted.

**Stack base:** Rust + Tauri v2, inferencia local con `ort` (ONNX Runtime) o `candle`,
llamadas a APIs externas para modelos pesados.

---

## Índice

1. [Image Generation](#1-image-generation)
2. [Image-to-Video](#2-image-to-video)
3. [Background Removal](#3-background-removal)
4. [Pixel Art Conversion](#4-pixel-art-conversion)
5. [SFX Generation](#5-sfx-generation)
6. [Music Generation](#6-music-generation)
7. [Speech / Voice](#7-speech--voice)
8. [PBR Materials](#8-pbr-materials)
9. [AI Coding Agent](#9-ai-coding-agent)
10. [Outpainting](#10-outpainting)
11. [Seamless Textures](#11-seamless-textures)
12. [Modelos de IA Confirmados en Sorceress.games](#12-modelos-de-ia-confirmados-en-sorceressgames)
13. [Resumen de Recomendaciones](#resumen-de-recomendaciones)
14. [Cambios Clave vs. Abril 2025](#cambios-clave-vs-abril-2025)

---

## 1. Image Generation

#### APIs de Pago (Estado del Arte — Abril 2026)

| Modelo | Proveedor | Precio | Calidad | Notas para game dev |
|--------|-----------|--------|---------|---------------------|
| **FLUX.2 [max]** | Black Forest Labs (fal.ai / BFL API) | $0.07/img | 5/5 | Mejor calidad del mercado; resoluciones hasta 2048px; ideal sprites HD y concept art |
| **FLUX.2 [pro]** | Black Forest Labs | $0.03/img | 4.5/5 | Equilibrio calidad/precio; buena adherencia al prompt |
| **FLUX.2 [flex]** | Black Forest Labs | $0.06/img | 4.5/5 | Soporta fine-tuning propio (LoRA upload); ideal para estilos custom por juego |
| **FLUX.2 [klein] 9B** | Black Forest Labs | desde $0.015/img | 4/5 | Precio por megapixel; excelente para batch generation de assets |
| **FLUX.2 [klein] 4B** | Black Forest Labs | desde $0.014/img | 3.5/5 | Modelo ligero; latencia baja; util para previews |
| **FLUX.1 Kontext [pro]** | Black Forest Labs | $0.04/img | 4/5 | Especializado en image editing in-context; inpainting y outpainting de alta calidad |
| **FLUX.1 Kontext [max]** | Black Forest Labs | $0.08/img | 4.5/5 | Version max de Kontext; mejor para ediciones complejas de sprites y backgrounds |
| **GPT-image-1.5** | OpenAI | $8/1M img-tokens input; $32/1M output | 4.5/5 | Excelente siguiendo instrucciones complejas; bueno para concept art narrativo |
| **Luma Photon** | Luma Labs | $0.0073/Mpx | 4/5 | Buena relacion calidad/precio; API limpia; facil integrar en Rust con reqwest |
| **Luma Photon Flash** | Luma Labs | $0.0019/Mpx | 3.5/5 | Rapido y barato; ideal previews y iteraciones rapidas |

> **Nota:** FLUX.2 reemplaza a FLUX.1 como modelo top de pago. FLUX.2 usa pricing por
> megapixel en las variantes [klein]; calcular costes según resolución objetivo del asset.
> FLUX.1 Kontext es la herramienta clave para workflows de edición iterativa de sprites.

#### Open Source / Self-Hosted

| Modelo | HuggingFace | Params | VRAM mín | Calidad | Licencia Comercial |
|--------|-------------|--------|----------|---------|-------------------|
| **FLUX.1 [schnell]** | `black-forest-labs/FLUX.1-schnell` | 12B | 12 GB | 4/5 | **Apache 2.0** — 1-4 pasos; mejor opción OSS para producción |
| **FLUX.1 [schnell] GGUF Q4** | `city96/FLUX.1-schnell-gguf` | 12B | 8 GB | 3.5/5 | Apache 2.0 — cuantizado; funciona en GPUs consumer |
| **FLUX.1 [dev]** | `black-forest-labs/FLUX.1-dev` | 12B | 16 GB | 4.5/5 | No comercial (FLUX.1-dev Non-Commercial) |
| **SD 3.5 Large** | `stabilityai/stable-diffusion-3.5-large` | 8B (MMDiT) | 16 GB | 4/5 | Restringida (hasta $1M revenue gratis; enterprise con acuerdo) |
| **SD 3.5 Medium** | `stabilityai/stable-diffusion-3.5-medium` | 2.5B | 8 GB | 3.5/5 | Restringida — buen balance para hardware mid-range |
| **SDXL Base 1.0** | `stabilityai/stable-diffusion-xl-base-1.0` | 3.5B | 8 GB | 3.5/5 | Sí (CreativeML OpenRAIL+M) |

**Notas técnicas:**
- FLUX.1 schnell: rectified flow transformer, convergencia en 1-4 pasos, no necesita negative prompt. Sigue siendo la mejor opción OSS comercial a abril 2026.
- FLUX.1 dev: guidance distillation, mejor calidad que schnell pero licencia no comercial.
- SD 3.5 Large usa triple encoder (CLIP-G + CLIP-L + T5-XXL); requiere 16 GB VRAM mínimo.
- Para pipelines locales en Rust: FLUX.1-schnell via candle o ONNX Runtime es viable con 12 GB VRAM.

### Algoritmos Clásicos
- **Procedural sprites**: noise-based (Perlin, Simplex), L-systems, Wave Function Collapse.
- **Palette generation**: median cut, k-means clustering, octree quantization.
- **Sprite composition**: layered blending, palettization + dithering (Floyd-Steinberg).

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `image` (image-rs) | Lectura/escritura PNG/JPEG/WebP/TIFF |
| `imageproc` | Filtros, transformaciones, convoluciones |
| `palette` | Espacios de color (sRGB, HSL, Lab, Oklch) |
| `ort` | ONNX Runtime — inferencia local de modelos exportados |
| `candle` (HuggingFace) | Framework ML en Rust puro, soporte CUDA |
| `diffusers-rs` | Bindings de diffusers para Rust (experimental) |

---

## 2. Image-to-Video

#### APIs de Pago (Estado del Arte — Abril 2026)

| Modelo | Proveedor | Precio | Calidad | Notas para game dev |
|--------|-----------|--------|---------|---------------------|
| **Sora 2** | OpenAI | $0.10/s (720p); $0.30/s (720p Pro); $0.50/s (1024p); $0.70/s (1080p) | 5/5 | Lanzado sept-2025; audio sincronizado nativo; física realista; ideal cinematics y trailers |
| **Runway Gen-4.5** | Runway | Plan Standard $12/mes; Pro $28/mes; Unlimited $76/mes | 4.5/5 | Modelo más reciente de Runway; alta consistencia temporal; ideal cutscenes |
| **Runway Gen-4** | Runway | Incluido en planes Runway | 4/5 | Generación robusta; buena control de cámara |
| **Veo 3** | Google / Runway | Incluido en plan Runway Standard+ | 5/5 | Generación de video con audio nativo; disponible en Runway desde abril 2026 |
| **Veo 3.1** | Google / Runway | Incluido en plan Runway Standard+ | 5/5 | Version actualizada de Veo 3; disponible en Runway |
| **Seedance 2.0** | ByteDance / Runway | Incluido en plan Runway | 4.5/5 | Disponible globalmente via Runway desde abril 2026; muy alta calidad de movimiento |
| **Luma Ray 2** | Luma Labs | $0.0064/Mpx (~$0.71 por 5s 720p) | 4.5/5 | API limpia; buena calidad; fácil integrar en Rust |
| **Luma Ray Flash 2** | Luma Labs | $0.0022/Mpx (~$0.24 por 5s 720p) | 3.5/5 | Versión rápida y económica; ideal para previews de animación |
| **Runway Act-Two** | Runway | Incluido en planes Runway | 4/5 | Performance capture; animar personajes desde video de referencia |

> **Nota:** Sora 2 es el líder en calidad pero el más caro; justificado para trailers y cinematics
> finales. Veo 3/3.1 y Seedance 2.0 disponibles vía Runway simplifican la integración (una sola API).
> Ray 2 de Luma ofrece la mejor API pública directa con pricing transparente por megapixel.

#### Open Source / Self-Hosted

| Modelo | HuggingFace | Tamaño | VRAM | Calidad | Licencia Comercial |
|--------|-------------|--------|------|---------|-------------------|
| **CogVideoX-5B** | `THUDM/CogVideoX-5b` | 5B | 16 GB | 4/5 | **Apache 2.0** |
| **CogVideoX-2B** | `THUDM/CogVideoX-2b` | 2B | 10 GB | 3.5/5 | Apache 2.0 |
| **Wan2.1** | `Wan-AI/Wan2.1-T2V-14B` | 14B | 24 GB | 4.5/5 | Apache 2.0 — excelente balance calidad/peso |
| **AnimateDiff v3** | `guoyww/animatediff-motion-adapter-v1-5-3` | ~3 GB | 8 GB | 3/5 | Apache 2.0 |
| **SVD XT** | `stabilityai/stable-video-diffusion-img2vid-xt` | ~10 GB | 24 GB | 3.5/5 | Stability AI (revisar) |
| **Open-Sora** | `hpcai-tech/Open-Sora` | ~7 GB | 24 GB | 3/5 | Apache 2.0 |

**Notas:**
- **CogVideoX-5B**: text-to-video, hasta 49 frames, 480×720. Mejor calidad open source mantenida.
- **AnimateDiff**: se inyecta como motion adapter en SD1.5/SDXL; 16-32 frames, muy flexible.
- **SVD XT**: img2vid, 25 frames a 576×1024, ~180 s en A100 80 GB — lento pero calidad visual alta.
- **Wan2.1**: modelo de enero 2025, excelente balance calidad/peso con 14B parámetros.

### Algoritmos Clásicos
- **Sprite animation**: keyframe interpolation, tweening (linear, cubic Bézier).
- **Skeletal animation**: forward/inverse kinematics, blend trees.
- **Procedural motion**: spring-damper `x'' = -k·x - c·x'`, Perlin noise sobre posiciones.

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `bevy_animation` | Sistema de animación (blend trees, clips) |
| `rapier2d` / `rapier3d` | Física de cuerpo rígido, articulaciones |
| `ozz-animation` | Bindings Rust para ozz-animation (skeletal, streaming) |
| `gltf` | Parser glTF 2.0 con animaciones |

---

## 3. Background Removal

#### APIs de Pago (Estado del Arte — Abril 2026)

| Proveedor | Precio | Calidad | Notas |
|-----------|--------|---------|-------|
| **fal.ai Background Removal API** | ~$0.001-0.005/imagen | 4/5 | Servicio gestionado sobre modelos OSS; util si no se quiere gestionar inferencia local |
| **fal.ai** (RMBG-2.0) | ~$0.005/imagen | 5/5 | Más barato con calidad RMBG |
| **remove.bg** | $0.10-$0.20/imagen | 4.5/5 | Especializado en personas/cabello |
| **BRIA AI** | Contacto | 5/5 | Licencia comercial de RMBG-2.0 |
| **Photoroom** | Suscripción $10/mes | 4.5/5 | Incluye otras herramientas |
| **Clipdrop** (Stability) | Suscripción | 4/5 | Integrado en suite Stability |

> **Nota complementaria:** Los LLMs multimodales (GPT-5.4 Vision, Claude Sonnet 4.6 Vision,
> Gemini 2.5 Pro Vision) son útiles para clasificación y descripción de assets, no para
> segmentación pixel-level. Para segmentación interactiva y extracción de sprites, SAM 2.1
> sigue siendo la referencia OSS.

#### Open Source / Self-Hosted

| Modelo | HuggingFace | Params | Tamaño | VRAM | Calidad | Licencia Comercial |
|--------|-------------|--------|--------|------|---------|-------------------|
| **SAM 2.1 Large** | `facebook/sam2-hiera-large` | ~310M | ~900 MB | 4 GB | 5/5 | **Apache 2.0** — versión más reciente (SAM 3 no existe a abril 2026) |
| **SAM 2.1 Base+** | `facebook/sam2-hiera-base-plus` | ~80M | ~300 MB | 2 GB | 4.5/5 | **Apache 2.0** |
| **BiRefNet** | `ZhengPeng7/BiRefNet` | ~100M | ~350 MB | 4 GB | 4.5/5 | **MIT** |
| **BiRefNet-lite** | `ZhengPeng7/BiRefNet-lite` | ~50M | ~180 MB | 2 GB | 4/5 | **MIT** |
| **BEN2** | GitHub | ~200M | ~150 MB | 2 GB | 4.5/5 | Ver licencia | 
| **RMBG-2.0** | `briaai/RMBG-2.0` | ~200M | ~100 MB | 2 GB | 5/5 | CC BY-NC 4.0 — **no comercial** sin acuerdo con BRIA |
| **rembg** (BiRefNet integrado) | PyPI: `rembg` | — | ~200 MB | CPU | 4/5 | Apache 2.0 — background removal local con BiRefNet integrado |
| **U2-Net** | GitHub: `xuebinqin/U-2-Net` | 44M | ~176 MB | 2 GB (CPU ok) | 3.5/5 | Apache 2.0 |

**Notas:**
- **SAM 2.1**: superior para segmentación interactiva (puntos/cajas como prompt) y tracking en video. SAM 3 no existe a abril 2026.
- **RMBG-2.0**: usa arquitectura BiRefNet con datos propietarios de BRIA. Excelente en gaming y e-commerce, pero requiere acuerdo comercial.
- **BiRefNet (MIT)**: mejor opción para producción comercial sin restricciones. Resultados comparables a RMBG-2.0 en imágenes generales.
- **rembg con BiRefNet**: integrable en Rust via ONNX Runtime (crate `ort`) o Python sidecar.
- Para batch processing automático sin interacción: BiRefNet-lite o SAM 2.1 Base+ con prompt automático.

### Algoritmos Clásicos
- **Chroma keying**: umbral en espacio HSV para eliminar fondo de color uniforme.
- **GrabCut** (OpenCV): segmentación semi-automática con iteraciones de GMM.
- **Flood fill**: relleno desde semilla + conectividad 4/8.
- **Alpha matting**: Closed-form matting, KNN matting.

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `ort` + ONNX export de BiRefNet | Inferencia local |
| `imageproc` | Post-proceso de máscaras (morfología, blur) |
| `opencv-rust` | GrabCut, flood fill, operaciones morfológicas |

---

## 4. Pixel Art Conversion

### Algoritmos Clásicos (sin IA)

**Color Quantization:**
| Algoritmo | Complejidad | Calidad | Notas |
|-----------|-------------|---------|-------|
| **Median Cut** | O(n log n) | 3.5/5 | Divide espacio de color recursivamente; clásico para GIF/PNG-8 |
| **k-means** | O(n·k·i) | 4/5 | Mejor calidad visual, más lento; i = iteraciones |
| **Wu's Algorithm** | O(n) | 4/5 | Rápido, baja memoria; usado en muchos encoders |
| **Octree** | O(n) | 3.5/5 | Muy eficiente, estructura jerárquica en el espacio de color |
| **NeuQuant** | O(n) | 4/5 | Red neuronal competitiva; usado en GIF animado |

**Downscale + Palette Pipeline:**
```
1. Downscale con nearest-neighbor (preserva bordes duros)
2. Color quantization (k-means o Wu) → paleta de N colores (8, 16, 32, 64)
3. Floyd-Steinberg dithering (distribución de error para mejor percepción)
4. Opcional: outline rendering (Sobel + thresholding)
```

**Dithering:**
- **Floyd-Steinberg**: distribución de error a vecinos (derecha, abajo-izquierda, abajo, abajo-derecha).
- **Ordered dithering (Bayer matrix)**: patrón fijo, menos artefactos en movimiento.
- **Atkinson dithering**: variante usada en Mac 128k, menor spread de error.

**Edge Detection para outline:**
- Sobel operator + thresholding.
- Canny edge detection (two-pass, non-maximum suppression).

### Modelos IA Especializados

| Modelo | Fuente | Notas |
|--------|--------|-------|
| `nerijs/pixel-art-xl` | HuggingFace LoRA | LoRA sobre SDXL para estilo pixel art |
| `PublicPrompts/All-In-One-Pixel-Model` | HuggingFace | Checkpoint SD 1.5 pixel art |
| `Reve-Anon/4xNomosUni_esrgan_multijpg` | HuggingFace | ESRGAN 4× para upscale pixel art |
| **PixelOE** | GitHub | Pixel art extraction desde foto, estado del arte 2024 |

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `image` | Resize con `FilterType::Nearest` (preserva píxeles) |
| `palette` | Conversión de espacios de color (sRGB↔Lab para cuantización perceptual) |
| `quantette` | Cuantización de color en Rust (Wu's algorithm, k-means) |
| Implementación custom k-means | Fácil en Rust: Vec<[u8;3]>, iteración hasta convergencia |

---

## 5. SFX Generation

#### APIs de Pago (Estado del Arte — Abril 2026)

| Proveedor | Precio | Calidad | Notas |
|-----------|--------|---------|-------|
| **ElevenLabs Sound Effects** | Incluido en plan (Starter $6/mes con LC) | 4.5/5 | Genera SFX desde descripción de texto; útil para footsteps, impactos, UI sounds |
| **Stability AI Audio API** | desde $0.012/seg | 4/5 | Stable Audio via API |
| **Freesound API** | Gratuito | N/A | Librería de SFX con licencias CC |
| **Soundly** | Suscripción | N/A | Biblioteca de SFX profesional |

> **Nota:** ElevenLabs consolida SFX, Music, TTS y Voice Cloning en una sola plataforma.
> Para game dev, una sola integración cubre todo el audio del juego desde el plan Starter ($6/mes).

#### Open Source / Self-Hosted

| Modelo | HuggingFace | Tamaño | VRAM | Calidad | Licencia Comercial |
|--------|-------------|--------|------|---------|-------------------|
| **Stable Audio Open 1.0** | `stabilityai/stable-audio-open-1.0` | ~2 GB | 8 GB | 4/5 | Stability AI (gratis hasta $1M revenue) |
| **AudioLDM 2** | `cvssp/audioldm2` | ~2.5 GB | 8 GB | 3.5/5 | **Apache 2.0** |
| **AudioLDM 2-Full** | `cvssp/audioldm2-full` | ~3.5 GB | 10 GB | 4/5 | **Apache 2.0** |
| **Tango 2** | `declare-lab/tango2` | ~2 GB | 8 GB | 3.5/5 | **Apache 2.0** |
| **AudioGen Medium** | `facebook/audiogen-medium` | 1.5B | 8 GB | 3/5 | CC BY-NC 4.0 — no comercial |
| **Bark** | `suno-ai/bark` | ~6 GB | 8 GB | 3.5/5 | **MIT** (SFX vía prompts de texto con sonidos especiales) |

**Notas:**
- **Stable Audio Open**: genera hasta 47 s de audio estéreo a 44.1 kHz — ideal para SFX de juegos. Latent diffusion sobre representación VAE de audio.
- **AudioLDM 2**: condicionado por texto, bueno para SFX descriptivos ("footsteps on gravel", "explosion"). Apache 2.0 es la gran ventaja.
- **Bark**: TTS + SFX via códigos especiales `[laughter]`, `[sighs]`, `[background noise]`; MIT pero pesado (~6 GB).

### Algoritmos Clásicos
- **Síntesis procedural**: osciladores (saw, square, sine), ADSR envelope, reverb/delay.
- **Síntesis FM (Frequency Modulation)**: Yamaha DX7-style, ideal para SFX retro.
- **Síntesis de ruido**: White noise + filtros pasa-banda para explosiones, viento, fuego.
- **Granular synthesis**: fragmentos cortos de audio reensamblados para texturas.

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `cpal` | Cross-platform Audio Library — playback/captura |
| `rodio` | Audio playback de alto nivel sobre cpal |
| `dasp` | Digital Audio Signal Processing — DSP primitives |
| `fundsp` | Síntesis de audio procedural (gráfo de señal) |
| `hound` | Lectura/escritura WAV |
| `symphonia` | Decodificación MP3/OGG/FLAC/AAC |

---

## 6. Music Generation

#### APIs de Pago (Estado del Arte — Abril 2026)

| Proveedor | Precio | Calidad | Notas |
|-----------|--------|---------|-------|
| **Suno v4** | Plan Basic $8/mes (500 canciones); Pro $24/mes (2500); Premier $96/mes (10000) | 4.5/5 | Excelente para OSTs de juego; generación completa con letra o instrumental |
| **ElevenLabs Music** | Incluido en plan (Starter $6/mes con LC) | 4/5 | Generación de música de fondo; controlable por mood y estilo; LC desde Starter |
| **Udio v1** | Plan Basic $10/mes; Pro $30/mes | 4/5 | Alternativa sólida a Suno; buena variedad de géneros |
| **Stability AI Audio** | Variable | 4/5 | Stable Audio via API |

#### Open Source / Self-Hosted

| Modelo | HuggingFace | Params | VRAM | Calidad | Licencia Comercial |
|--------|-------------|--------|------|---------|-------------------|
| **MusicGen Large** | `facebook/musicgen-large` | 3.3B | 16 GB | 4/5 | CC BY-NC 4.0 — **no comercial** |
| **MusicGen Medium** | `facebook/musicgen-medium` | 1.5B | 8 GB | 3.5/5 | CC BY-NC 4.0 — **no comercial** |
| **MusicGen Melody** | `facebook/musicgen-melody` | 1.5B | 8 GB | 3.5/5 | CC BY-NC 4.0 — **no comercial** |
| **Stable Audio Open** | `stabilityai/stable-audio-open-1.0` | ~2 GB | 8 GB | 3.5/5 | Stability AI (ver licencia) |
| **MusicLM** | Solo paper (Google, no release público) | — | — | 4.5/5 | No disponible |

**Notas:**
- **MusicGen**: genera hasta ~30 s, 32 kHz mono, entrenado con datos de Meta/Shutterstock/Pond5. Límite principal: CC BY-NC = sin uso comercial gratuito.
- **Stable Audio Open**: mejor para loops cortos con BPM conocido (47 s estéreo a 44.1 kHz). Prompts como "128 BPM, game loop, synth, upbeat".
- No existe a abril 2026 un modelo open-source de música de calidad comparable a Suno/Udio con licencia Apache 2.0. El gap sigue siendo significativo.

### Algoritmos Clásicos
- **MIDI procedural**: generación de melodías con Markov chains, reglas de voz.
- **Tracker music** (MOD/XM): síntesis basada en samples + patrones.
- **WaveNet-style vocoder**: síntesis de audio desde features acústicas.
- **Adaptive music**: sistemas de capas (Wwise/FMOD-style) en lugar de generación IA.

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `midly` | Parse/write MIDI |
| `nodi` | MIDI playback |
| `fundsp` | Síntesis procedural de audio |
| `cpal` + `rodio` | Playback en tiempo real |

---

## 7. Speech / Voice

#### APIs de Pago (Estado del Arte — Abril 2026)

| Proveedor | Precio | Calidad | Notas |
|-----------|--------|---------|-------|
| **ElevenLabs TTS** | Créditos del plan (Starter $6/mes con LC; Pro $99/mes 600k créditos 192kbps) | 5/5 | Líder en calidad de voz; voces emocionales; ideal NPCs y narradores |
| **ElevenLabs Voice Cloning** | Instant desde Starter; Professional desde Creator ($22/mes) | 4.5/5 | Clonar voces de NPCs; excelente para personalización de personajes |
| **gpt-realtime-1.5** | OpenAI | $32/1M audio input tokens; $64/1M audio output tokens | 4.5/5 | Voz en tiempo real con latencia muy baja; util para NPCs interactivos con diálogo generativo |
| **OpenAI TTS** | OpenAI | $15/1M caracteres (tts-1-hd) | 4/5 | TTS de alta calidad; más barato que gpt-realtime para contenido pregrabado |
| **Cartesia** | $0.015/min | 4.5/5 | <100 ms (streaming); muy baja latencia |
| **Azure TTS** | $4/1M chars (neural) | 4/5 | 200 ms de latencia |
| **PlayHT** | $0.015/1K chars | 4/5 | 400 ms de latencia |
| **Deepgram Aura** | $0.0150/1K chars | 3.5/5 | <200 ms; económico |

> **Nota:** gpt-realtime-1.5 es la opción para NPCs con diálogo hablado en tiempo real (no
> pregrabado). ElevenLabs cubre TTS pregrabado, clonación de voces y SFX desde una sola
> integración; LC comercial activa desde el plan Starter ($6/mes).

#### Open Source / Self-Hosted

| Modelo | HuggingFace | Tamaño | VRAM | Calidad | Licencia Comercial |
|--------|-------------|--------|------|---------|-------------------|
| **Kokoro-82M** | `hexgrad/Kokoro-82M` | 82M (~330 MB) | 1 GB (CPU ok) | 4/5 | **Apache 2.0** — MEJOR opción libre para TTS con calidad |
| **Parler-TTS Mini v1** | `parler-tts/parler-tts-mini-v1` | 880M | 4 GB | 3.5/5 | **Apache 2.0** |
| **F5-TTS** | `SWivid/F5-TTS` | ~300 MB | 2 GB | 4/5 | CC BY-NC 4.0 — no comercial |
| **Fish Speech 1.5** | `fishaudio/fish-speech-1.5` | ~1 GB | 4 GB | 4.5/5 | CC-BY-NC-SA-4.0 — no comercial |
| **XTTS v2** | `coqui/XTTS-v2` | ~1.8 GB | 4 GB (CPU lento) | 4.5/5 | Coqui Public License — no comercial sin pagar |
| **Bark** | `suno-ai/bark` | ~6 GB | 8 GB | 3.5/5 | **MIT** — calidad irregular, lento |
| **MeloTTS** | `myshell-ai/MeloTTS-English` | ~370 MB | 2 GB (CPU ok) | 3.5/5 | **MIT** |

**Notas clave para producción comercial:**
- **Kokoro-82M (Apache 2.0)**: calidad sorprendente para 82M parámetros. Principalmente inglés. Latencia ~50-100 ms en CPU moderna. La opción por defecto para producción libre.
- **F5-TTS**: voice cloning con sólo 10 s de audio de referencia, flow matching, 4/5 calidad. Licencia CC-BY-NC impide uso comercial directo.
- **Fish Speech 1.5**: 13 idiomas, entrenado con 1M horas, excelente multilingüe. Licencia NC.
- **XTTS v2**: 17 idiomas, clonación con 6 s de referencia. Licencia Coqui no comercial por defecto (tier comercial ~$149/mes).

### Algoritmos Clásicos
- **TTS concatenativo**: síntesis por unidades (difonos), sin IA, latencia <10 ms.
- **Formant synthesis**: síntesis de voz robótica clásica (Festival TTS).
- **Prosody rules**: análisis del texto → entonación, pausas, énfasis.

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `ort` + ONNX export | Inferencia de modelos TTS (Kokoro exportado a ONNX) |
| `cpal` / `rodio` | Playback de audio generado |
| `hound` | WAV I/O |
| `rubato` | Resampling de alta calidad |

---

## 8. PBR Materials

#### APIs de Pago (Estado del Arte — Abril 2026)

| Proveedor | Precio | Calidad | Notas |
|-----------|--------|---------|-------|
| **Meshy 5** | Free: 100 créditos/mes; Pro: 1000 cr; Studio: 4000 cr | 4.5/5 | Version actual de Meshy; plugins para Blender, Unity, Unreal, **Godot**, Maya, 3DS Max, Roblox; assets privados desde Pro |
| **Tripo v3.0 Ultra** | Professional $11.94/mes (3000 cr); Advanced $29.94/mes (8000 cr); Premium $83.94/mes (25000 cr) | 4.5/5 | Alta calidad de malla; clientes verificados: Tencent, Sony, HTC |
| **Stability AI (API)** | $0.065/gen | 4/5 | Normal/depth generation |
| **Adobe Substance 3D** | $49.99/mes | 5/5 | Suite completa PBR, industry standard |
| **Poly.cam** | $20/mes | 4/5 | Photogrammetry + PBR desde foto |
| **Luma Genie** | En desarrollo | 4.5/5 | Text-to-3D con PBR |

> **Nota:** Meshy 5 destaca por su integración directa con motores de juego (plugin Godot
> disponible). El plan Free (100 créditos/mes) permite evaluar la calidad sin coste inicial.
> Para PBR materials desde imagen: FLUX.1 Kontext sigue siendo una opción eficaz para generar
> albedo/roughness/normal maps iterativamente vía edición in-context.

### Modelos IA Open Source

| Modelo | Tipo | Calidad | Licencia | Notas |
|--------|------|---------|----------|-------|
| **MaterialFuse** | Paper (2024, no release público) | 5/5 | N/A | Fusión de materiales PBR desde imagen; estado del arte en investigación |
| **DreamMat** | GitHub (threedle/DreamMat) | 4/5 | MIT | Text-to-PBR material, basado en SD + SDS loss |
| **MatFormer** | Paper (2023) | 4/5 | No release | Genera maps PBR desde foto única |
| **StableProjectorz** | HuggingFace | 3.5/5 | Apache 2.0 | Genera normal/roughness desde diffuse |
| **ControlNet (tile/depth)** | `lllyasviel/ControlNet-v1-1` | 3.5/5 | Apache 2.0 | Hints de profundidad/normal para SD |
| **InstructPix2Pix** | `timbrooks/instruct-pix2pix` | 3/5 | Apache 2.0 | Edición de maps vía instrucción de texto |
| **Shap-E** | OpenAI / HuggingFace | 3/5 | MIT | Generación 3D rápida; calidad limitada; útil para prototipado rápido de props |
| **Point-E** | OpenAI / HuggingFace | 2.5/5 | MIT | Más rápido que Shap-E pero menor calidad |

**Pipeline típico para PBR completo desde prompt:**
```
1. Generar Albedo (diffuse) → FLUX.1-schnell / FLUX.2 [pro]
2. Estimar Depth → isl-org/ZoeDepth o Depth-Anything-V2
3. Derivar Normal Map desde depth (gradiente de Sobel 3D)
4. Roughness/Metallic: heurísticas desde albedo (luminance → roughness inversa)
5. AO (Ambient Occlusion): SSAO en shader o estimación desde depth
6. Preview PBR en tiempo real con wgpu / bevy_pbr
```

### Outputs PBR (formato estándar)
| Map | Canales | Descripción |
|-----|---------|-------------|
| **Albedo** | RGB | Color base sin iluminación directa |
| **Normal** | RGB | Perturbación de normales en espacio tangente |
| **Roughness** | R | Rugosidad specular (0=espejo, 1=difuso) |
| **Metallic** | R | Flujo PBR metallic (0=dieléctrico, 1=metal) |
| **AO** | R | Oclusión ambiental |
| **Emissive** | RGB | Auto-iluminación |
| **Height** | R | Desplazamiento de malla (tessellation/parallax) |

### Algoritmos Clásicos para PBR
- **Normal from Height**: Sobel 3D sobre heightmap → vector (dx, dy, dz).
- **SSAO**: Screen Space Ambient Occlusion desde depth buffer.
- **Parallax mapping**: desplazamiento UV basado en heightmap (no tessellation).
- **Procedural roughness**: ruido Perlin sobre superficie → variación de roughness.
- **Texture bombing**: distribución aleatoria de detalles para evitar repetición.

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `wgpu` | Preview PBR en tiempo real (WebGPU/Vulkan/Metal) |
| `bevy_pbr` | Sistema PBR completo de Bevy (albedo, normal, roughness, metallic) |
| `ort` | Inferencia de modelos de estimación depth/normal |
| `image` | Manipulación de maps (canales, normalización) |
| `glam` | Operaciones vectoriales/matriciales para normal computation |

---

## 9. AI Coding Agent

#### APIs de Pago (Estado del Arte — Abril 2026)

| Proveedor | Modelo | Precio (input/output por 1M tokens) | Calidad | Notas |
|-----------|--------|-------------------------------------|---------|-------|
| **OpenAI** | GPT-5.4 Pro | $30 / $180 | 5/5 | Máximo razonamiento; para tareas de arquitectura compleja en Rust/WASM |
| **OpenAI** | GPT-5.4 | $2.50 / $15 | 4.5/5 | Balance calidad/precio; context window grande; buen coding en Rust |
| **OpenAI** | GPT-5.4 Mini | $0.75 / $4.50 | 4/5 | Rápido y económico; util para generación de código boilerplate |
| **OpenAI** | GPT-5.4 Nano | $0.20 / $1.25 | 3.5/5 | Muy barato; tareas simples de completion y templates |
| **Anthropic** | Claude Opus 4.6 | $5 / $25 | 5/5 | Context 200K; excelente para entender codebases grandes; DDD y arquitectura |
| **Anthropic** | Claude Sonnet 4.6 | $3 / $15 | 4.5/5 | Mejor relación calidad/precio de Anthropic; ideal coding assistant diario |
| **Anthropic** | Claude Haiku 4.5 | $1 / $5 | 4/5 | Rápido y barato; bueno para tareas repetitivas de codegen |
| **DeepSeek** | DeepSeek-V3.2 | $0.28 / $0.42 (cache miss); $0.028 input (cache hit) | 4.5/5 | El más barato del mercado con calidad top; modo thinking integrado; lanzado dic-2025 |
| **Google** | Gemini 2.5 Pro | $1.25 / $10 (hasta 200K); $2.50 / $15 (>200K) | 4.5/5 | Context 1M tokens; excelente para analizar codebases muy grandes |
| **Google** | Gemini 2.5 Flash | $0.15 / $0.60 | 4/5 | Muy económico; bueno para tareas de coding frecuentes |
| **Mistral** | Codestral | $0.30 / $0.90 | 4/5 | Especializado en código |

#### Open Source / Self-Hosted

| Modelo | HuggingFace | Params | VRAM | Calidad Código | Licencia Comercial |
|--------|-------------|--------|------|----------------|-------------------|
| **Llama 4 Maverick (17B/128E MoE)** | Meta | 24 GB | 4.5/5 | Sí (Llama 4 Community) | Versión avanzada; compite con modelos cloud de primera línea |
| **Llama 4 Scout (17B/16E MoE)** | Meta | 16 GB | 4/5 | Sí (Llama 4 Community) | MoE 17B activos; buena calidad coding; self-hosteable |
| **Qwen3 72B** | `Qwen/Qwen3-72B-Instruct` | 48 GB | 4.5/5 | **Apache 2.0** — excelente código; multilingüe |
| **Qwen3 14B** | `Qwen/Qwen3-14B-Instruct` | 16 GB | 4/5 | **Apache 2.0** — buen balance para hardware mid-range |
| **Qwen2.5-Coder-32B** | `Qwen/Qwen2.5-Coder-32B-Instruct` | 32.5B | 24 GB BF16 / 16 GB int4 | 4.5/5 | **Apache 2.0** — referencia anterior; Qwen3 lo supera en coding |
| **Mistral Medium 3** | Mistral | API / OSS | ~$0.40 / $2 | 4/5 | Sí — modelo europeo con garantías de privacidad |
| **DeepSeek-V3** | `deepseek-ai/DeepSeek-V3` | 671B total / 37B activos (MoE) | Cluster multi-GPU | 5/5 | Sí (DeepSeek Model License v1) — inferencia local solo viable en cluster |
| **CodeLlama-34B** | `codellama/CodeLlama-34b-Instruct-hf` | 34B | 20 GB | 3.5/5 | Meta Llama 2 License (sí comercial) |
| **StarCoder2-15B** | `bigcode/starcoder2-15b` | 15B | 10 GB | 3.5/5 | BigCode OpenRAIL-M (sí comercial) |

**Notas:**
- **DeepSeek-V3.2**: el más barato del mercado con calidad top; 10x más barato que Claude Sonnet 4.6; modo thinking integrado elimina la necesidad de modelos separados reasoning/non-reasoning.
- **Claude Opus 4.6**: con 200K context es ideal para trabajar con el codebase completo de Sorceress en Rust.
- **GPT-5.4 Pro**: se justifica solo para tareas de arquitectura de alto nivel; muy caro para uso diario.
- **Qwen3 72B**: con Apache 2.0 es la mejor opción self-hosted si se dispone de 48 GB VRAM (ej: 2x RTX 3090). Supera a Qwen2.5-Coder-32B.
- **Qwen2.5-Coder-32B**: sigue siendo referencia histórica para single-GPU, pero Qwen3 14B lo supera a menor VRAM.

### Algoritmos Clásicos (Asistencia de Código sin IA)
- **Static analysis**: AST parsing, control/data flow analysis (rust-analyzer, clippy).
- **Fuzzy search**: Tree-sitter queries, ripgrep, semgrep.
- **Code completion**: n-gram language models, Aho-Corasick para snippets.

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `reqwest` | HTTP client para APIs de LLM |
| `async-openai` | Cliente async para OpenAI-compatible APIs |
| `ort` | Inferencia local con modelos ONNX |
| `llama-cpp-rs` | Bindings para llama.cpp (GGUF models) |
| `candle` | Framework ML Rust puro con soporte cuantización |
| `tree-sitter` | Parsing incremental de código (análisis AST) |

---

## 10. Outpainting

#### APIs de Pago (Estado del Arte — Abril 2026)

| Proveedor | Precio | Calidad | Notas |
|-----------|--------|---------|-------|
| **FLUX.1 Kontext [pro]** (fal.ai / BFL) | $0.04/imagen | 4/5 | Nuevo paradigma para edición in-context; inpainting y outpainting de alta fidelidad |
| **FLUX.1 Kontext [max]** (fal.ai / BFL) | $0.08/imagen | 4.5/5 | Mejor para ediciones complejas de sprites y backgrounds |
| **fal.ai** (FLUX.1 Fill pro) | ~$0.05/imagen | 5/5 | Inpainting clásico con FLUX.1 Fill |
| **Stability AI** (inpaint) | $0.065/imagen | 4/5 | |
| **Replicate** (SDXL Inpaint) | ~$0.02/imagen | 3.5/5 | |
| **OpenAI** (DALL-E 3 inpaint) | $0.040/imagen | 4/5 | |

> **Nota:** FLUX.1 Kontext introduce un nuevo paradigma para workflows de edición iterativa
> de sprites y backgrounds: permite editar regiones específicas con alta coherencia contextual
> sin necesidad de máscaras explícitas en muchos casos de uso.

#### Open Source / Self-Hosted

| Modelo | HuggingFace | Tipo | Calidad | Licencia Comercial |
|--------|-------------|------|---------|-------------------|
| **FLUX.1 Fill [dev]** | `black-forest-labs/FLUX.1-Fill-dev` | OS | 5/5 | No comercial (FLUX.1-dev NC) |
| **FLUX.1 Fill [pro]** | API via BFL/fal.ai | API | 5/5 | Sí (API) — ~$0.05/imagen |
| **SD 3.5 Large Inpaint** | `stabilityai/stable-diffusion-3.5-large` | OS | 4/5 | Stability AI license |
| **SDXL Inpainting** | `diffusers/stable-diffusion-xl-1.0-inpainting-0.1` | OS | 3.5/5 | **Apache 2.0** |
| **SD Inpainting v2.1** | `stabilityai/stable-diffusion-2-inpainting` | OS | 3/5 | CreativeML OpenRAIL |
| **ControlNet Inpaint** | `lllyasviel/control_v11p_sd15_inpaint` | OS | 3.5/5 | **Apache 2.0** (sobre SD1.5) |

**Notas:**
- **FLUX.1 Fill**: el estado del arte en inpainting/outpainting clásico (dic 2024). 12B parámetros, arquitectura FLUX.1 con masked conditioning.
- **FLUX.1 Kontext**: nuevo paradigma (2025); edición in-context sin necesidad de máscaras explícitas en muchos casos.
- **SDXL Inpainting + Apache 2.0**: la mejor opción comercial self-hosted. Menos calidad que FLUX.1 Fill pero libre para producción.
- **Outpainting iterativo**: expandir imagen por partes (overlapping patches de 512 píxeles con 64-128 píxeles de overlap) para resoluciones arbitrarias.

### Técnicas de Outpainting

| Técnica | Complejidad | Calidad | Notas |
|---------|-------------|---------|-------|
| **Masked diffusion** | — | 4.5/5 | Máscara binaria define región a rellenar |
| **Context-aware editing** | — | 5/5 | Edición in-context sin máscara (FLUX.1 Kontext) |
| **Tiled outpainting** | O(n_tiles) | 4/5 | Expansión iterativa con overlap |
| **Context-aware fill** | — | 5/5 | Modelo infiere desde bordes (FLUX.1 Fill) |
| **Seam carving** | O(w·h) | 3/5 | Content-aware resizing no-IA (Avidan & Shamir 2007) |
| **Inpainting clásico** | O(patch·n) | 2.5/5 | PatchMatch (Adobe), síntesis de textura |

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `image` | Padding, compositing, canvas expansion |
| `ort` | Inferencia local de modelos de inpainting (ONNX) |
| `imageproc` | Blending de bordes, suavizado de transiciones |

---

## 11. Seamless Textures

### Modelos IA

| Modelo | Fuente | Calidad | Licencia | Notas |
|--------|--------|---------|----------|-------|
| **FLUX.1 schnell + prompts** | HuggingFace | 3.5/5 | Apache 2.0 | "seamless texture, tileable, [style]" |
| **ControlNet Tile** | `lllyasviel/control_v11f1e_sd15_tile` | 4/5 | Apache 2.0 | Hace tileable cualquier imagen via SD |
| **SD Outpainting + wrap** | diffusers | 3.5/5 | Apache 2.0 | Wrap + outpainting iterativo para hacer seamless |
| **Dreamlike Photoreal 2.0** | `dreamlike-art/dreamlike-photoreal-2.0` | 4/5 | Dreamlike license | Checkpoint óptimo para texturas realistas |
| **Material Diffusion** | GitHub (tencent-ailab) | 4/5 | Apache 2.0 | Genera texturas PBR tileable desde texto |

**Pipeline IA para textura seamless:**
```
1. Generar imagen base con FLUX/SD usando prompt "seamless, tileable texture"
2. Aplicar "mirror padding trick": espejo en todos los bordes antes de pasarla por SD
3. Usar ControlNet Tile para refinar coherencia
4. Verificar tileability: pegar 2×2 copias y comprobar sin artefactos en bordes
```

### Algoritmos Clásicos

**Wang Tiles:**
- Conjunto de tiles con bordes coloreados codificados.
- La regla de Wang: tiles adyacentes deben compartir color de borde.
- Garantiza seamless sin repetición visible.
- Complejidad: O(n) para generar mapa de tiles.
- Conjunto mínimo: 4 tiles de Wang de 2 colores; conjunto de 16 tiles para patrones ricos.

**Wave Function Collapse (WFC):**
- Basado en constraint propagation (AC-3).
- Genera tilesets coherentes localmente a partir de ejemplos.
- Complejidad: O(n·k) donde k = número de patrones posibles.
- Implementaciones: `wfc` crate Rust, `fast-wfc`, `wavefunctioncollapse` (original C++).
- Variantes: overlapping model (aprendizaje de patrones N×N) vs. simple tiled model.

**Seamless Blending:**
| Técnica | Descripción | Calidad |
|---------|-------------|---------|
| **Cross-fading** | Blend lineal en los bordes (fade de 10-20% del ancho) | 2.5/5 |
| **Poisson blending** | Gradiente de Poisson para mezcla coherente (OpenCV seamlessClone) | 4/5 |
| **Frequency domain** | FFT → eliminar frecuencias de borde → IFFT | 3.5/5 |
| **Histogram matching** | Igualar histogramas de tiles adyacentes | 3/5 |

**Procedural Textures (sin IA ni imagen de entrada):**
- **Perlin/Simplex noise**: base para casi todo (madera, mármol, nubes).
- **Voronoi diagrams**: patrones celulares, piedra, tierra agrietada.
- **FBM (Fractional Brownian Motion)**: octavas de noise sumadas.
- **Domain warping**: input warpings recursivos para formas orgánicas complejas.
- Todas son seamless por definición si se usan en coordenadas de tiling modular.

### Librerías Rust
| Crate | Rol |
|-------|-----|
| `image` | Operaciones de tile, wrapping, mirroring |
| `noise` | Perlin, Simplex, Worley (Voronoi), FBM |
| `wgpu-noise` | Generación de noise en shader (GPU) |
| `fast-wfc` | Port Rust de Wave Function Collapse |
| `imageproc` | Blending, filtros de borde |
| `opencv-rust` | Poisson blending (seamlessClone) |

---

## 12. Modelos de IA Confirmados en Sorceress.games

> **Metodologia**: Datos extraidos del codigo JavaScript de produccion de `sorceress.games` (Next.js chunks).
> Certeza alta para todos los datos marcados con confirmado.

### Arquitectura de APIs

Sorceress no llama directamente a todos los proveedores. Usa **Kie.ai** como capa intermediaria
de API (mas barata que fal.ai) para imagen, video, musica y SFX. Para imagen tambien tiene fallback
a **Replicate** directamente. El coding agent usa **BYO API Key** (Bring Your Own Key) — el usuario
proporciona su propia API key.

### Image Generation — 9 modelos

| Modelo | Proveedor real | Certeza |
|--------|---------------|---------|
| Grok Imagine | xAI | Confirmado |
| GPT Image 1.5 | OpenAI | Confirmado |
| Nano Banana / 2 / Pro | Google (interno) | Confirmado |
| Seedream 4.5 / 5 Lite | ByteDance | Confirmado |
| Flux 2 Pro / Flux Schnell | Black Forest Labs | Confirmado |
| Imagen 4 Ultra | Google (solo Tileset) | Confirmado |

Image-to-Image: flux-2-pro, seedream-5-lite, grok-imagine, nano-banana-edit.

### Video Generation — 7 modelos (Image-to-Video)

| Modelo | Proveedor real | Certeza |
|--------|---------------|---------|
| Kling 2.5 Turbo Pro | Kwai (KlingAI) | Confirmado |
| Wan 2.7 / Wan 2.2 Fast | Alibaba/Wan | Confirmado |
| Seedance 1.5 Pro / 2.0 / 2.0 Fast | ByteDance | Confirmado |
| Grok Imagine Video | xAI | Confirmado |

### Music Generation

| Modelo | Proveedor real | Certeza |
|--------|---------------|---------|
| Suno V4/V4.5 | Suno AI (via Kie.ai) | Confirmado |

Genera 2 variaciones por peticion. Soporta letras y prompts de estilo.

### SFX Generation

| Modelo | Proveedor real | Certeza |
|--------|---------------|---------|
| Kie Sounds v5.5 | Kie.ai (propio o wrapped ElevenLabs) | Confirmado |

Costo: 3 creditos por generacion.

### Speech / Voice Generation

| Modelo | Proveedor real | Certeza |
|--------|---------------|---------|
| MiniMax Speech 2.8 (Speech-02 HD) | MiniMax (hailuo) | Confirmado |

Soporta voice cloning y expresiones emocionales. Voces preset + personalizadas.
**Dato clave**: NO usa ElevenLabs para voz — usa MiniMax.

### AI Coding Agent (BYO API Key)

| Modelo | Proveedor | Precio (input/output por M tokens) |
|--------|-----------|-------------------------------------|
| Claude Opus 4.6 (default) | Anthropic | $15 / $75 |
| DeepSeek Reasoner | DeepSeek | $0.55 / $2.19 |
| GPT-5 Nano | OpenAI | $0.05 / $0.40 |
| GPT-5.2 Codex | OpenAI | $1.75 / $14 |
| Kimi K2.5 (NVIDIA) | Moonshot AI via NVIDIA | Gratis (free trial) |

### Quick Sprites (Pixel Art Animado)

Usa los mismos modelos de imagen + **Retro Diffusion RD Animation** (Retro Diffusion).

### Tileset Generator

Modelos dedicados: GPT Image 1.5, Seedream 5 Lite, Imagen 4 Ultra.

### Herramientas sin IA externa (client-side / server-side propio)

| Herramienta | Tecnologia |
|-------------|-----------|
| Auto-Sprite / Spritely | Chroma key + canvas (navegador) |
| True Pixel (pixel art) | Cuantizacion de color + dithering |
| SFX Editor | Web Audio API |
| Slicer | Canvas API |
| 3D to 2D | Three.js (WebGL) |
| Batch Utilities | FFmpeg.wasm + sharp.js |

### Resumen de Proveedores

| Proveedor | Features | Rol |
|-----------|----------|-----|
| **Kie.ai** | Image, Video, Music (Suno), SFX | API hub intermediario |
| **Replicate** | Image (fallback) | API directa |
| **OpenAI** | GPT Image 1.5, GPT-5 Nano/Codex | API directa |
| **Anthropic** | Claude Opus 4.6 (coding) | API directa (BYO key) |
| **Google** | Nano Banana, Imagen 4 Ultra | Via Kie.ai |
| **ByteDance** | Seedream (img), Seedance (video) | Via Kie.ai |
| **xAI** | Grok Imagine (img + video) | Via Kie.ai |
| **Black Forest Labs** | Flux 2 Pro, Flux Schnell | Via Kie.ai |
| **Kwai/KlingAI** | Kling 2.5 (video) | Via Kie.ai |
| **Alibaba/Wan** | Wan 2.7 (video) | Via Kie.ai |
| **MiniMax** | Speech-02-HD (TTS) | API directa |
| **Moonshot AI** | Kimi K2.5 (coding) | Via NVIDIA |
| **DeepSeek** | DeepSeek Reasoner (coding) | API directa (BYO key) |
| **Retro Diffusion** | RD Animation (pixel art) | Via Kie.ai |

### Implicaciones para nuestro clon

1. **Kie.ai como API hub**: Considerar usar un hub de APIs similar en vez de integraciones directas
   con cada proveedor. Simplifica el billing y el routing de modelos.
2. **BYO API Key para coding**: El usuario pone su propia key. Esto elimina el coste de LLM para Sorceress
   y evita problemas de privacidad con el codigo del usuario.
3. **MiniMax para TTS** (no ElevenLabs): Sorceress eligio MiniMax sobre ElevenLabs para voz.
   Posibles razones: mejor precio, mejor voice cloning, o integracion mas simple.
4. **Google Nano Banana**: Modelos internos de Google no disponibles publicamente. Sorceress tiene
   acceso probablemente via partnership con Google/Kie.ai. Para nuestro clon, usar FLUX.2 como
   alternativa equivalente.
5. **Herramientas client-side**: Auto-Sprite, Pixel Art, Slicer y Batch Utils corren enteramente
   en el navegador. Para Tauri v2, estas serian modulos Rust nativos con mejor rendimiento.

---

## Resumen de Recomendaciones

### Para Producción Comercial

#### Tier 1 — Self-hosted totalmente libre (Apache 2.0 / MIT)

| Categoría | Modelo | VRAM | Calidad |
|-----------|--------|------|---------|
| Imagen | FLUX.1-schnell | 12 GB | 4/5 |
| Video | CogVideoX-5B o Wan2.1-14B | 16-24 GB | 4-4.5/5 |
| Background removal | BiRefNet (MIT) | 4 GB | 4.5/5 |
| Segmentación interactiva | SAM 2.1 Large | 4 GB | 5/5 |
| TTS | Kokoro-82M | CPU ok | 4/5 |
| Coding agent local | Qwen3 72B (o Qwen3 14B) | 48 GB (16 GB) | 4.5/5 |
| SFX | AudioLDM 2 | 8 GB | 3.5/5 |
| Pixel art | Algoritmos clásicos | — | 4/5 |
| Outpainting | SDXL Inpainting | 8 GB | 3.5/5 |
| Texturas seamless | noise + WFC | — | 4/5 |
| 3D prototipado | Shap-E | 8 GB | 3/5 |

#### Tier 2 — APIs más cost-effective

| Categoría | Proveedor | Precio aprox. |
|-----------|-----------|---------------|
| Imagen | FLUX.2 [pro] (fal.ai / BFL) | $0.03/imagen |
| Video | Luma Ray 2 | ~$0.71 por 5s 720p |
| TTS / SFX / Music | ElevenLabs Starter | $6/mes (cubre todo el audio) |
| Coding | DeepSeek-V3.2 API | $0.28/$0.42 per 1M tokens |
| Background removal | fal.ai Background Removal API | ~$0.003/imagen |
| Outpainting | FLUX.1 Kontext [pro] (fal.ai) | $0.04/imagen |
| 3D assets | Meshy 5 Free tier | 100 créditos/mes gratis |

#### Tier 3 — Máxima calidad (precio mayor)

| Categoría | Proveedor | Precio |
|-----------|-----------|--------|
| Imagen | FLUX.2 [max] (BFL) | $0.07/imagen |
| Video | Sora 2 (OpenAI) | $0.10-$0.70/s según resolución |
| TTS / Voice | ElevenLabs Pro | $99/mes |
| Coding | Claude Sonnet 4.6 (Anthropic) | $3/$15 per 1M tokens |
| Coding máx. calidad | GPT-5.4 Pro (OpenAI) | $30/$180 per 1M tokens |
| Music | Suno v4 Basic | $8/mes |
| 3D assets game-ready | Meshy 5 Pro (plugin Godot) | 1000 créditos/mes |
| PBR suite completa | Adobe Substance 3D | $49.99/mes |

---

### Decisiones de Arquitectura Clave

1. **FLUX.2 como modelo top de imagen de pago, FLUX.1-schnell como mejor OSS local**: FLUX.2 [pro] ($0.03/img) para producción en cloud; FLUX.1-schnell (Apache 2.0, 12 GB VRAM) para inferencia local. A abril 2026 siguen siendo las referencias en sus respectivos segmentos.

2. **FLUX.1 Kontext como nuevo paradigma para edición de sprites**: Permite inpainting y outpainting in-context con alta fidelidad, cambiando el workflow de edición iterativa. Referencia para backgrounds y sprites generados con correcciones sucesivas.

3. **BiRefNet (MIT) para background removal**: Sin royalties, calidad comparable a soluciones comerciales. SAM 2.1 para segmentación interactiva. SAM 3 no existe a abril 2026.

4. **Kokoro-82M para TTS**: Apache 2.0, corre en CPU, 82M parámetros con calidad sorprendente. La opción por defecto para producción libre.

5. **ElevenLabs como suite unificada de audio**: Una sola integración en Rust cubre SFX, Music, TTS y Voice Cloning. Licencia comercial activa desde Starter ($6/mes). Elimina la necesidad de múltiples SDKs de audio.

6. **DeepSeek-V3.2 con thinking integrado como coding backbone económico**: 10x más barato que Claude Sonnet 4.6 con calidad comparable. El modo thinking integrado elimina la necesidad de modelos separados reasoning/non-reasoning. Ideal para alto volumen de generación de código.

7. **AudioLDM 2 para SFX local**: Apache 2.0, condicionado por texto, ideal para "footsteps on gravel" etc. Sin música OSS comercialmente libre de calidad — usar Suno/Udio API o síntesis procedural (MIDI + samples) como alternativa.

8. **Meshy 5 con plugin Godot para 3D**: El workflow más directo para generar 3D assets en pipelines de game dev. Plan Free (100 créditos/mes) para evaluación.

9. **WFC + noise procedural para texturas seamless**: Sin dependencia de IA, determinista, escalable.

10. **ort (ONNX Runtime) como motor de inferencia en Rust**: Compatible con PyTorch/JAX exportado a ONNX, soporte GPU via CUDA/TensorRT. Aplica a BiRefNet, SAM 2.1, Kokoro-82M y otros modelos locales.

---

## Cambios Clave vs. Abril 2025

1. **FLUX.2 reemplaza a FLUX.1 como modelo top de imagen de pago**: Black Forest Labs lanzó FLUX.2 entre noviembre 2025 y enero 2026 con cuatro variantes [max/pro/flex/klein]. FLUX.1-schnell sigue siendo la mejor opción OSS comercial para uso local.

2. **FLUX.1 Kontext es nuevo paradigma para edición de imagen**: Permite edición in-context (inpainting, outpainting, style transfer) con alta fidelidad; cambia el workflow de edición iterativa de sprites respecto al FLUX.1 Fill anterior.

3. **Sora 2 lanzado septiembre 2025 con audio nativo**: El salto cualitativo respecto a Sora 1 es significativo; incluye física realista y audio sincronizado; disponible en API.

4. **Video generation democratizado vía Runway**: Veo 3/3.1 (Google) y Seedance 2.0 (ByteDance) disponibles en Runway desde abril 2026, ofreciendo acceso a los mejores modelos de video a través de una sola plataforma/API.

5. **Claude 4.x reemplaza Claude 3.x en todas las variantes**: Opus 4.6 / Sonnet 4.6 / Haiku 4.5 son los modelos actuales; precios similares pero calidad muy superior, especialmente en coding en Rust y análisis de codebases grandes.

6. **GPT-5.4 reemplaza GPT-4o**: La familia GPT-5.4 (nano/mini/base/pro) es la línea actual de OpenAI. GPT-image-1.5 es el modelo de imagen actual (no DALL-E ni GPT-image-1).

7. **DeepSeek-V3.2 mantiene liderazgo en precio**: Lanzado diciembre 2025; sigue siendo 10x más barato que competidores con calidad comparable; ahora incluye modo thinking integrado.

8. **Meshy 5 es la referencia en 3D para game dev**: Con plugins para los 6 principales motores/herramientas (Blender, Unity, Unreal, Godot, Maya, Roblox), Meshy 5 es el workflow más directo para generar 3D assets en pipelines de game dev.

9. **ElevenLabs consolida suite completa de audio**: La integración de SFX, Music, TTS, STT y Voice Cloning en una sola plataforma y API simplifica el stack de audio para game dev. Licencia comercial disponible desde el plan Starter ($6/mes).

10. **SAM 2.1 es la versión más reciente de SAM**: SAM 3 no existe a abril 2026. SAM 2.1 sigue siendo el estado del arte en segmentación interactiva de imagen/video; integrable en Rust vía ONNX Runtime.

---

*Actualizado: Abril 2026*
*Proyecto: Sorceress — Suite de herramientas de Game Dev con IA*
*Stack: Rust + Tauri v2*
*Próxima revisión recomendada: octubre 2026*
