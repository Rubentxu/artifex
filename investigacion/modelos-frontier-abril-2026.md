# Modelos IA Frontier para Game Dev — Abril 2026

**Propósito:** Actualización del estado del arte de modelos IA relevantes para la suite Sorceress
(game dev con IA), cubriendo modelos publicados o actualizados significativamente entre
abril 2025 y abril 2026.

**Referencia anterior:** `modelos-ia-game-tools.md` (cubre hasta abril 2025)

**Stack base proyecto:** Rust + Tauri v2, inferencia local con `ort` / `candle`,
llamadas a APIs externas para modelos pesados.

**Columnas de tablas:**
- **Disponibilidad**: API (solo cloud) | OSS (open weights) | Ambos
- **Precio**: tarifa al corte de abril 2026, sin IVA, USD
- **Calidad**: 1–5 (criterio: calidad de output para uso en game dev)
- **LC**: Licencia Comercial (Si / No / Restringida)

---

## Índice

1. [Image Generation](#1-image-generation)
2. [Video Generation](#2-video-generation)
3. [Audio — SFX, Music y Voice](#3-audio--sfx-music-y-voice)
4. [Coding / LLM](#4-coding--llm)
5. [3D Generation / PBR](#5-3d-generation--pbr)
6. [Vision / Background Removal](#6-vision--background-removal)
7. [Recomendaciones por Caso de Uso](#7-recomendaciones-por-caso-de-uso)
8. [Cambios Clave vs. Abril 2025](#8-cambios-clave-vs-abril-2025)

---

## 1. Image Generation

### APIs de Pago (cloud-only o con tier comercial)

| Modelo | Proveedor | Disponibilidad | Precio | Calidad | LC | Notas para game dev |
|--------|-----------|---------------|--------|---------|-----|---------------------|
| **FLUX.2 [max]** | Black Forest Labs | API (fal.ai, BFL API) | $0.07/img | 5/5 | Si | Mejor calidad del mercado; resoluciones hasta 2048px; ideal sprites HD y concept art |
| **FLUX.2 [pro]** | Black Forest Labs | API | $0.03/img | 4.5/5 | Si | Equilibrio calidad/precio; buena adherencia al prompt |
| **FLUX.2 [flex]** | Black Forest Labs | API | $0.06/img | 4.5/5 | Si | Soporta fine-tuning propio (LoRA upload); ideal para estilos custom por juego |
| **FLUX.2 [klein] 9B** | Black Forest Labs | API | desde $0.015/img | 4/5 | Si | Precio basado en megapixeles; excelente para batch generation de assets |
| **FLUX.2 [klein] 4B** | Black Forest Labs | API | desde $0.014/img | 3.5/5 | Si | Modelo ligero; latencia baja; util para previews |
| **FLUX.1 Kontext [pro]** | Black Forest Labs | API | $0.04/img | 4/5 | Si | Especializado en image editing in-context; inpainting y outpainting de alta calidad |
| **FLUX.1 Kontext [max]** | Black Forest Labs | API | $0.08/img | 4.5/5 | Si | Version max de Kontext; mejor para ediciones complejas de sprites y backgrounds |
| **GPT-image-1.5** | OpenAI | API | $8/1M img-tokens input; $32/1M output | 4.5/5 | Si | Siguiendo instructions complejas; bueno para concept art narrativo |
| **Photon** | Luma Labs | API | $0.0073/Mpx | 4/5 | Si | Buena relacion calidad/precio; API limpia; facil integrar en Rust con reqwest |
| **Photon Flash** | Luma Labs | API | $0.0019/Mpx | 3.5/5 | Si | Rapido y barato; ideal previews y iteraciones rapidas |

### Open Source / Self-hosted

| Modelo | HuggingFace / Fuente | VRAM min | Calidad | LC | Notas para game dev |
|--------|---------------------|----------|---------|-----|---------------------|
| **FLUX.1 [schnell]** | `black-forest-labs/FLUX.1-schnell` | 12 GB | 4/5 | Si (Apache 2.0) | Sigue siendo la mejor opcion OSS comercial; 1-4 pasos; sin negative prompt |
| **FLUX.1 [schnell] GGUF Q4** | `city96/FLUX.1-schnell-gguf` | 8 GB | 3.5/5 | Si (Apache 2.0) | Cuantizado; funciona en GPUs consumer; compatible con llama.cpp/candle |
| **SD 3.5 Large** | `stabilityai/stable-diffusion-3.5-large` | 16 GB | 4/5 | Restringida | Hasta $1M revenue sin fee; triple encoder CLIP-G + CLIP-L + T5-XXL |
| **SD 3.5 Medium** | `stabilityai/stable-diffusion-3.5-medium` | 8 GB | 3.5/5 | Restringida | Buen balance para hardware mid-range; 2.5B params |

**Notas tecnicas:**
- FLUX.2 usa pricing por megapixel; calcular costes segun resolucion objetivo del asset.
- FLUX.1 Kontext es la herramienta clave para workflows de edicion iterativa de sprites.
- GPT-image-1.5 destaca en adherencia a instrucciones complejas; mas caro que FLUX.2.
- Para pipelines locales en Rust: FLUX.1-schnell via candle o ONNX Runtime es viable con 12 GB VRAM.

---

## 2. Video Generation

### APIs de Pago

| Modelo | Proveedor | Disponibilidad | Precio | Calidad | LC | Notas para game dev |
|--------|-----------|---------------|--------|---------|-----|---------------------|
| **Sora 2** | OpenAI | API | $0.10/s (720p); $0.30/s (720p Pro); $0.50/s (1024p); $0.70/s (1080p) | 5/5 | Si | Lanzado sept-2025; audio sincronizado nativo; fisica realista; ideal cinematics y trailers |
| **Gen-4.5** | Runway | API / Web | Plan Standard $12/mes; Pro $28/mes; Unlimited $76/mes | 4.5/5 | Si | Modelo mas reciente de Runway; alta consistencia temporal; ideal cutscenes |
| **Gen-4** | Runway | API / Web | Incluido en planes Runway | 4/5 | Si | Generacion robusta; buena control de camara |
| **Veo 3** | Google / Runway | API via Runway (plan Standard+) | Incluido en plan Runway Standard+ | 5/5 | Si | Generacion de video con audio nativo; disponible en Runway desde abril 2026 |
| **Veo 3.1** | Google / Runway | API via Runway | Incluido en plan Runway Standard+ | 5/5 | Si | Version actualizada de Veo 3; disponible en Runway |
| **Seedance 2.0** | ByteDance / Runway | API via Runway worldwide | Incluido en plan Runway | 4.5/5 | Si | Disponible globalmente via Runway desde abril 2026; muy alta calidad de movimiento |
| **Ray 2** | Luma Labs | API | $0.0064/Mpx (~$0.71 por 5s 720p) | 4.5/5 | Si | API limpia; buena calidad; facil integrar en Rust |
| **Ray Flash 2** | Luma Labs | API | $0.0022/Mpx (~$0.24 por 5s 720p) | 3.5/5 | Si | Version rapida y economica de Ray 2; ideal para previews de animacion |
| **Aleph** | Runway | API / Web | Incluido en planes Runway | 4/5 | Si | Especializado en video editing; util para post-procesado de cinematics |
| **Act-Two** | Runway | API / Web | Incluido en planes Runway | 4/5 | Si | Performance capture; animar personajes desde video de referencia |

**Notas tecnicas:**
- Sora 2 es el lider en calidad pero el mas caro; justificado para trailers y cinematics finales.
- Veo 3/3.1 y Seedance 2.0 disponibles via Runway, lo que simplifica la integracion (una sola API).
- Ray 2 de Luma ofrece la mejor API publica directa; pricing transparente por megapixel.
- Para game dev: video generation util para trailers, cutscenes pre-renderizadas, y referencias de animacion.

---

## 3. Audio — SFX, Music y Voice

### Plataformas Todo-en-Uno

| Modelo / Plataforma | Proveedor | Disponibilidad | Precio | Calidad | LC | Notas para game dev |
|--------------------|-----------|---------------|--------|---------|-----|---------------------|
| **ElevenLabs API** | ElevenLabs | API | Free: 10k creditos/mes; Starter $6/mes (30k, LC incluida); Creator $22/mes (121k); Pro $99/mes (600k, 192kbps) | 4.5/5 | Si (desde Starter) | Suite completa: TTS, STT, SFX, Music, Voice Design, Voice Cloning; una sola integracion para todo el audio del juego |
| **ElevenLabs Voice Cloning** | ElevenLabs | API | Instant desde Starter; Professional desde Creator | 4.5/5 | Si (desde Starter) | Clonar voces de NPCs; excelente para personalizacion de personajes |

### SFX Generation

| Modelo | Proveedor | Disponibilidad | Precio | Calidad | LC | Notas para game dev |
|--------|-----------|---------------|--------|---------|-----|---------------------|
| **ElevenLabs Sound Effects** | ElevenLabs | API (incluido en planes ElevenLabs) | Creditos del plan | 4.5/5 | Si | Genera SFX desde descripcion de texto; util para footsteps, impactos, UI sounds |
| **AudioCraft / AudioGen** | Meta | OSS | Libre | 3.5/5 | Restringida (CC-BY-NC) | Open source pero licencia no comercial; util para prototipado |

### Music Generation

| Modelo | Proveedor | Disponibilidad | Precio | Calidad | LC | Notas para game dev |
|--------|-----------|---------------|--------|---------|-----|---------------------|
| **ElevenLabs Music** | ElevenLabs | API (incluido en planes ElevenLabs) | Creditos del plan | 4/5 | Si (desde Starter, commercial use) | Generacion de musica de fondo; controlable por mood y estilo; LC desde Starter |
| **Suno v4** | Suno | API / Web | Plan Basic $8/mes (500 canciones); Pro $24/mes (2500); Premier $96/mes (10000) | 4.5/5 | Si (planes de pago) | Excelente para OSTs de juego; generacion completa con letra o instrumental |
| **Udio v1** | Udio | API / Web | Plan Basic $10/mes; Pro $30/mes | 4/5 | Si (planes de pago) | Alternativa solida a Suno; buena variedad de generos |

### Voice / TTS

| Modelo | Proveedor | Disponibilidad | Precio | Calidad | LC | Notas para game dev |
|--------|-----------|---------------|--------|---------|-----|---------------------|
| **ElevenLabs TTS** | ElevenLabs | API | Creditos del plan (Starter $6/mes con LC) | 5/5 | Si | Lider en calidad de voz; voces emocionales; ideal NPCs y narradores |
| **gpt-realtime-1.5** | OpenAI | API | $32/1M audio input tokens; $64/1M audio output tokens | 4.5/5 | Si | Voz en tiempo real con latencia muy baja; util para NPCs interactivos con dialogo generativo |
| **OpenAI TTS** | OpenAI | API | $15/1M caracteres (tts-1-hd) | 4/5 | Si | TTS de alta calidad; mas barato que gpt-realtime para contenido pregrabado |

**Notas tecnicas:**
- ElevenLabs es la plataforma mas eficiente para game dev: una sola integracion cubre SFX, Music, TTS y Voice Cloning.
- Suno v4 es el lider en generacion musical completa para OSTs.
- gpt-realtime-1.5 es la opcion para NPCs con dialogo hablado en tiempo real (no pregrabado).
- Licencia comercial en ElevenLabs se activa desde el plan Starter ($6/mes), no en el tier Free.

---

## 4. Coding / LLM

### Modelos Frontier via API

| Modelo | Proveedor | Disponibilidad | Precio (input/output por 1M tokens) | Calidad | LC | Notas para game dev |
|--------|-----------|---------------|-------------------------------------|---------|-----|---------------------|
| **GPT-5.4 Pro** | OpenAI | API | $30 / $180 | 5/5 | Si | Maximo razonamiento; para tareas de arquitectura compleja en Rust/WASM |
| **GPT-5.4** | OpenAI | API | $2.50 / $15 | 4.5/5 | Si | Balance calidad/precio; context window grande; buen coding en Rust |
| **GPT-5.4 Mini** | OpenAI | API | $0.75 / $4.50 | 4/5 | Si | Rapido y economico; util para generacion de codigo boilerplate |
| **GPT-5.4 Nano** | OpenAI | API | $0.20 / $1.25 | 3.5/5 | Si | Muy barato; tareas simples de completion y templates |
| **Claude Opus 4.6** | Anthropic | API | $5 / $25 | 5/5 | Si | Context 200K; excelente para entender codebases grandes; DDD y arquitectura |
| **Claude Sonnet 4.6** | Anthropic | API | $3 / $15 | 4.5/5 | Si | Mejor relacion calidad/precio de Anthropic; ideal coding assistant diario |
| **Claude Haiku 4.5** | Anthropic | API | $1 / $5 | 4/5 | Si | Rapido y barato; bueno para tareas repetitivas de codegen |
| **DeepSeek-V3.2** | DeepSeek | API | $0.28 / $0.42 (cache miss); $0.028 input cache hit | 4.5/5 | Si | El mas barato del mercado con calidad top; modo thinking integrado; lanzado dic-2025 |
| **Gemini 2.5 Pro** | Google | API | $1.25 / $10 (hasta 200K); $2.50 / $15 (>200K) | 4.5/5 | Si | Context 1M tokens; excelente para analizar codebases muy grandes |
| **Gemini 2.5 Flash** | Google | API | $0.15 / $0.60 | 4/5 | Si | Muy economico; bueno para tareas de coding frecuentes |

### Open Source / Self-hosted

| Modelo | HuggingFace / Fuente | VRAM min | Calidad | LC | Notas para game dev |
|--------|---------------------|----------|---------|-----|---------------------|
| **Llama 4 Scout (17B/16E MoE)** | Meta | 16 GB | 4/5 | Si (Llama 4 Community) | MoE 17B activos; buena calidad coding; self-hosteable |
| **Llama 4 Maverick (17B/128E MoE)** | Meta | 24 GB | 4.5/5 | Si (Llama 4 Community) | Versión avanzada; compite con modelos cloud de primera línea |
| **Qwen3 72B** | Alibaba / HuggingFace | 48 GB | 4.5/5 | Si (Apache 2.0) | Excelente codigo; multilingue; apache 2.0 permite uso comercial sin restricciones |
| **Qwen3 14B** | Alibaba / HuggingFace | 16 GB | 4/5 | Si (Apache 2.0) | Buen balance para hardware mid-range |
| **Mistral Medium 3** | Mistral | API / OSS | API: ~$0.40 / $2 | 4/5 | Si | Buena relacion calidad/precio; modelo europeo con garantias de privacidad |

**Notas tecnicas:**
- DeepSeek-V3.2 es la opcion mas economica para alto volumen de generacion de codigo (10x mas barato que Claude Sonnet 4.6).
- Claude Opus 4.6 con 200K context es ideal para trabajar con el codebase completo de Sorceress en Rust.
- GPT-5.4 Pro se justifica solo para tareas de arquitectura de alto nivel; muy caro para uso diario.
- Para self-hosting: Qwen3 72B con Apache 2.0 es la mejor opcion si se dispone de 48 GB VRAM (ej: 2x RTX 3090).
- DeepSeek modo thinking integrado elimina la necesidad de modelos separados reasoning/non-reasoning.

---

## 5. 3D Generation / PBR

### APIs de Pago

| Modelo / Plataforma | Proveedor | Disponibilidad | Precio | Calidad | LC | Notas para game dev |
|--------------------|-----------|---------------|--------|---------|-----|---------------------|
| **Meshy 5** | Meshy | API / Web | Free: 100 creditos/mes; Pro: 1000 creditos; Studio: 4000 creditos. Assets privados desde Pro | 4.5/5 | Si (Pro+) | Version actual de Meshy; plugins para Blender, Unity, Unreal, Godot, Maya, 3DS Max, Roblox |
| **Tripo v3.0 Ultra** | Tripo3D | API / Web | Professional $11.94/mes (3000 cr); Advanced $29.94/mes (8000 cr); Premium $83.94/mes (25000 cr) | 4.5/5 | Si | Clientes verificados: Tencent, Sony, HTC, Bambu Lab; alta calidad de malla |

### Open Source / Self-hosted

| Modelo | HuggingFace / Fuente | VRAM min | Calidad | LC | Notas para game dev |
|--------|---------------------|----------|---------|-----|---------------------|
| **Shap-E** | OpenAI / HuggingFace | 8 GB | 3/5 | Si (MIT) | Generacion 3D rapida; calidad limitada; util para prototipado rapido de props |
| **Point-E** | OpenAI / HuggingFace | 6 GB | 2.5/5 | Si (MIT) | Mas rapido que Shap-E pero menor calidad |

**Notas tecnicas:**
- Meshy 5 destaca por su integracion directa con motores de juego (plugin Godot disponible); flujo de trabajo mas directo para game assets.
- Tripo v3.0 Ultra genera mallas de mayor calidad; preferible para assets finales de produccion.
- El plan Free de Meshy (100 creditos/mes) permite evaluar la calidad sin coste inicial.
- Para PBR materials desde imagen: FLUX.1 Kontext sigue siendo la mejor opcion (edicion de imagen para generar albedo/roughness/normal maps iterativamente).
- No existe aun un modelo OSS comparable en calidad a los servicios cloud para 3D generation.

---

## 6. Vision / Background Removal

| Modelo | Proveedor | Disponibilidad | Precio | Calidad | LC | Notas para game dev |
|--------|-----------|---------------|--------|---------|-----|---------------------|
| **SAM 2.1** | Meta | OSS | Libre | 4.5/5 | Si (Apache 2.0) | Version mas reciente (SAM 3 NO existe a abril 2026); segmentacion de video y imagen; excelente para sprite extraction |
| **GPT-5.4 Vision** | OpenAI | API | $2.50 / $15 (tokens de texto + imagen) | 5/5 | Si | Analisis de imagenes; util para sprite sheet analysis y descripcion de assets |
| **Claude Sonnet 4.6 Vision** | Anthropic | API | $3 / $15 | 4.5/5 | Si | Context 200K; puede analizar multiples imagenes a la vez; excelente para feedback de assets |
| **rembg (BiRefNet)** | OSS / PyPI | OSS | Libre | 4/5 | Si (MIT) | Background removal local; modelo BiRefNet integrado; ejecutable en Rust via Python sidecar o ONNX |
| **Background Removal API** | fal.ai | API | ~$0.001-0.005/imagen | 4/5 | Si | Servicio gestionado sobre modelos OSS; util si no se quiere gestionar inferencia local |
| **Gemini 2.5 Pro Vision** | Google | API | $1.25 / $10 | 4.5/5 | Si | Context 1M; puede procesar imagenes y video; util para analisis de game assets en bulk |

**Notas tecnicas:**
- SAM 2.1 (no SAM 3) es el estado del arte en segmentacion; ideal para extraer sprites de fondos complejos de forma interactiva.
- rembg con BiRefNet es la mejor opcion local para background removal automatico; integrable en Rust via ONNX Runtime (crate `ort`).
- Los LLMs multimodales (GPT-5.4 Vision, Claude Vision) son utiles para clasificacion y descripcion de assets, no para segmentacion pixel-level.

---

## 7. Recomendaciones por Caso de Uso

| Caso de Uso | Opcion Recomendada | Alternativa Economica | Opcion OSS Local |
|-------------|--------------------|-----------------------|-----------------|
| Generacion de sprites HD | FLUX.2 [pro] ($0.03/img) | FLUX.2 [klein] 9B ($0.015/img) | FLUX.1-schnell (12 GB VRAM) |
| Concept art narrativo | GPT-image-1.5 | FLUX.2 [max] | FLUX.1-schnell |
| Edicion / inpainting de sprites | FLUX.1 Kontext [pro] ($0.04/img) | FLUX.1 Kontext [max] ($0.08/img) | — |
| Generacion de backgrounds (estilos custom) | FLUX.2 [flex] + LoRA ($0.06/img) | FLUX.2 [pro] | FLUX.1-schnell + LoRA |
| Background removal de sprites | rembg (BiRefNet, local) | fal.ai API (~$0.003/img) | SAM 2.1 (interactivo) |
| Cinematics / trailers | Sora 2 (calidad maxima) | Runway Gen-4.5 | — |
| Animacion de personajes | Runway Act-Two | Luma Ray 2 | — |
| SFX del juego | ElevenLabs ($6/mes Starter) | ElevenLabs Sound Effects | AudioCraft (no comercial) |
| OST / musica de fondo | Suno v4 ($8/mes Basic) | ElevenLabs Music | — |
| Voces de NPCs pregrabadas | ElevenLabs TTS | OpenAI TTS | — |
| NPCs con dialogo en tiempo real | gpt-realtime-1.5 | GPT-5.4 Mini + TTS | — |
| Coding assistant (uso diario) | Claude Sonnet 4.6 ($3/$15) | DeepSeek-V3.2 ($0.28/$0.42) | Qwen3 14B (16 GB VRAM) |
| Coding (alto volumen / batch) | DeepSeek-V3.2 (mas barato) | GPT-5.4 Nano | Qwen3 72B (48 GB VRAM) |
| Arquitectura / codigo complejo | Claude Opus 4.6 (200K ctx) | GPT-5.4 Pro | Llama 4 Maverick |
| Generacion 3D de props | Meshy 5 (plugin Godot) | Tripo v3.0 Ultra | Shap-E (baja calidad) |
| Analisis bulk de game assets | Gemini 2.5 Pro Vision (1M ctx) | Claude Sonnet 4.6 Vision | SAM 2.1 (segmentacion) |

---

## 8. Cambios Clave vs. Abril 2025

1. **FLUX.2 reemplaza a FLUX.1 como modelo top de imagen**: Black Forest Labs lanzo FLUX.2 entre
   noviembre 2025 y enero 2026 con cuatro variantes [max/pro/flex/klein]. FLUX.1-schnell sigue
   siendo la mejor opcion OSS comercial para uso local.

2. **FLUX.1 Kontext es nuevo paradigma para edicion de imagen**: Permite edicion in-context
   (inpainting, outpainting, style transfer) con alta fidelidad; cambia el workflow de edicion
   iterativa de sprites.

3. **Sora 2 lanzado septiembre 2025 con audio nativo**: El salto cualitativo respecto a Sora 1
   es significativo; incluye fisica realista y audio sincronizado; disponible en API.

4. **Video generation democratizado via Runway**: Veo 3/3.1 (Google) y Seedance 2.0 (ByteDance)
   disponibles en Runway desde abril 2026, ofreciendo acceso a los mejores modelos de video a
   traves de una sola plataforma/API.

5. **Claude 4.x reemplaza Claude 3.x en todas las variantes**: Opus 4.6 / Sonnet 4.6 / Haiku 4.5
   son los modelos actuales; precios similares pero calidad muy superior, especialmente en coding
   en Rust y analisis de codebases grandes.

6. **GPT-5.4 reemplaza GPT-4o**: La familia GPT-5.4 (nano/mini/base/pro) es la linea actual de
   OpenAI. GPT-image-1.5 es el modelo de imagen actual (no DALL-E ni GPT-image-1).

7. **DeepSeek-V3.2 mantiene liderazgo en precio**: Lanzado diciembre 2025; sigue siendo 10x mas
   barato que competidores con calidad comparable; ahora incluye modo thinking integrado.

8. **Meshy 5 es la referencia en 3D para game dev**: Con plugins para los 6 principales motores/
   herramientas (Blender, Unity, Unreal, Godot, Maya, Roblox), Meshy 5 es el workflow mas directo
   para generar 3D assets en pipelines de game dev.

9. **ElevenLabs consolida suite completa de audio**: La integracion de SFX, Music, TTS, STT y
   Voice Cloning en una sola plataforma y API simplifica el stack de audio para game dev.
   Licencia comercial disponible desde el plan Starter ($6/mes).

10. **SAM 2.1 es la version mas reciente de SAM**: SAM 3 no existe a abril 2026. SAM 2.1 sigue
    siendo el estado del arte en segmentacion interactiva de imagen/video; integrable en Rust via
    ONNX Runtime.

---

*Documento generado: abril 2026. Proxima revision recomendada: octubre 2026.*
