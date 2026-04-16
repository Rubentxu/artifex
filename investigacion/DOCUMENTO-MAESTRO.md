# Documento de Ingeniería Inversa: Sorceress → Aplicación Rust + Tauri v2

> Versión 1.0 — Abril 2026
> Documento autocontenido de arquitectura, diseño y hoja de ruta para construir un clon funcional de Sorceress usando Rust, Tauri v2 y DDD.

---

## Tabla de Contenidos

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Inventario Completo de Features](#2-inventario-completo-de-features)
3. [Modelos de IA — Estado del Arte (Abril 2026)](#3-modelos-de-ia--estado-del-arte-abril-2026)
4. [Arquitectura del Sistema](#4-arquitectura-del-sistema)
5. [Stack Tecnológico Detallado](#5-stack-tecnológico-detallado)
6. [Flujo de Datos por Feature](#6-flujo-de-datos-por-feature)
7. [UX y Diseño de Interfaz](#7-ux-y-diseño-de-interfaz)
8. [Roadmap de Desarrollo](#8-roadmap-de-desarrollo)
9. [Riesgos y Mitigaciones](#9-riesgos-y-mitigaciones)
10. [Referencias](#10-referencias)

---

## 1. Resumen Ejecutivo

### Que es Sorceress

Sorceress es una suite de herramientas creativas con IA orientada a game developers independientes. Su propuesta de valor central es reunir en una sola aplicación desktop todas las tareas de producción de assets que normalmente requieren docenas de herramientas distintas: generación de imágenes, sprites, tilesets, materiales PBR, efectos de sonido, música, voz, y codificación asistida por agentes. El usuario objetivo es el desarrollador indie que trabaja solo o en equipos pequeños y necesita producir assets de calidad sin los recursos de un estudio grande.

La arquitectura de Sorceress combina procesamiento local (modelos de IA que corren en la GPU del usuario) con llamadas a APIs cloud de pago según la capacidad del hardware disponible. Esta dualidad local/cloud permite que la herramienta funcione en hardware modesto usando APIs externas y en hardware potente (RTX 3090/4090 o superior) usando modelos completamente locales. La monetización original de Sorceress mezcla un tier gratuito limitado con suscripciones Pro y créditos de uso para las operaciones que consumen más recursos.

Sorceress no es únicamente un wrapper de APIs: incorpora flujos de trabajo completos específicos para game dev, como el pipeline Auto-Sprite (video → spritesheet con metafile para Godot/Unity), la generación de tilesets seamless con Wang tiles, la creación de materiales PBR desde una sola imagen, y un coding agent que puede generar sistemas completos de código para motores de juego con preview en tiempo real. Esta especialización de dominio es lo que distingue a Sorceress de herramientas de IA genéricas como Midjourney o ChatGPT.

### Que vamos a construir

Vamos a construir una aplicación desktop multiplataforma (Windows, macOS, Linux) que replica la funcionalidad core de Sorceress usando un stack completamente open source y sin dependencias de licencias propietarias. La aplicación se llamará igual internamente en el código base. El frontend será una aplicación web embebida en Tauri v2 (usando React o SvelteKit), el backend será Rust puro con un workspace de crates separados por dominio, y el procesamiento pesado de IA estará delegado a workers independientes que se comunican con el proceso principal via IPC.

La estrategia de construcción sigue los principios de Domain-Driven Design: cada dominio funcional (sprites, imágenes, audio, video, tiles, materiales, agente de código) vive en su propio bounded context con su propio lenguaje ubicuo, sus propias entidades y sus propias reglas de negocio. El proceso principal de Tauri actúa como Control Plane (orquestación, UI, base de datos) y los workers son el Data Plane (procesamiento pesado, inferencia de modelos). Esta separación garantiza que un fallo en la inferencia de un modelo no derrumbe la aplicación entera.

Para los modelos de IA se priorizarán modelos open source con licencias Apache 2.0 o MIT para máxima libertad comercial: FLUX.1-schnell para imagen, BiRefNet para background removal, AudioLDM 2 para SFX, Kokoro-82M para TTS, y Qwen3 14B para coding. Cuando la calidad local no sea suficiente o el hardware del usuario no soporte los modelos, el sistema fallback automáticamente a APIs de pago (FLUX.2 API, ElevenLabs (suite unificada), DeepSeek-V3.2 API, Claude Sonnet 4.6) con costos transparentes para el usuario.

### Stack tecnológico elegido y por qué

| Capa | Tecnología | Razón de la elección |
|------|------------|----------------------|
| Framework desktop | Tauri v2 (Rust) | Binarios de 2-5 MB vs 150 MB de Electron; seguridad por defecto; Rust nativo |
| Frontend | React + Vite o SvelteKit | Ecosistema robusto; SvelteKit compila a JS vanilla sin runtime |
| UI components | shadcn/ui + Tailwind CSS | Componentes copiables sin vendor lock-in; accesibilidad; dark mode nativo |
| Canvas | PixiJS (sprite viewer) + Konva.js (editor interactivo) | WebGL para performance; Canvas2D para edición con layers |
| State management | Zustand (React) / Svelte Stores | Mínimo boilerplate; sin Redux overhead |
| Backend Rust | Workspace con crates por dominio | Separación de concerns; compilación incremental; sin dependencias cruzadas |
| Inferencia local | candle (HuggingFace) + ort (ONNX Runtime) | Pure Rust; soporte CUDA/Metal; compatible con safetensors y GGUF |
| LLM grandes | llama.cpp como sidecar | Mejor optimización de cuantización GGUF para modelos 7B-70B |
| GPU compute | wgpu | Cross-platform; WebGPU compatible; sin dependencia CUDA directa para shaders |
| Base de datos | SQLite via sqlx | Persistencia local sin servidor; SQL expresivo; migraciones; ACID |
| HTTP server local | axum | Ergonómico; tokio nativo; usado por los workers para exponer REST API |
| IPC workers | Unix Domain Sockets / Named Pipes | Sin overhead de red; framing JSON-lines; simple de implementar |

La elección de Tauri frente a Electron no es ideológica sino práctica: el binario base de Tauri ocupa ~3 MB frente a ~150 MB de Electron, el uso de memoria es significativamente menor porque usa el WebView nativo del sistema operativo, y Rust proporciona garantías de memoria en tiempo de compilación que reducen drásticamente las clases de bugs en producción.

### Estimación de esfuerzo

| Fase | Semanas | Desarrolladores | Complejidad |
|------|---------|-----------------|-------------|
| Fase 0: Fundaciones | 1-2 | 1-2 | Media |
| Fase 1: Core Tools (sin IA) | 3-6 | 2 | Alta |
| Fase 2: IA Integration | 7-10 | 2-3 | Muy Alta |
| Fase 3: Coding Agent | 11-14 | 2-3 | Muy Alta |
| Fase 4: Publishing y Polish | 15-18 | 2 | Media |
| **Total** | **18 semanas** | **2-3 personas** | — |

Para un desarrollador solo con experiencia en Rust y React/Svelte, multiplicar el tiempo estimado por 1.5x (27 semanas). Para un equipo de 4+, las fases 2 y 3 pueden ejecutarse en paralelo reduciendo a ~14 semanas.

---

## 2. Inventario Completo de Features

La siguiente tabla compila todas las features identificadas en Sorceress, organizadas por categoría funcional. Las categorías corresponden directamente a los bounded contexts del sistema (ver sección 4).

| # | Feature | Categoría | Tipo | Complejidad | Prioridad |
|---|---------|-----------|------|-------------|-----------|
| 1 | **Auto-Sprite** | Sprite Factory | Pro | Alta | P0 |
| 2 | **Sprite Sheet Slicer** | Sprite Factory | Free | Media | P0 |
| 3 | **Pixel Art Converter** | Sprite Factory | Pro | Media | P1 |
| 4 | **3D to 2D Renderer** | Sprite Factory | Pro | Alta | P2 |
| 5 | **Image Generation** | Image Engine | Credit | Alta | P0 |
| 6 | **Background Removal** | Image Engine | Pro | Media | P0 |
| 7 | **Outpainting / Image Expansion** | Image Engine | Credit | Alta | P1 |
| 8 | **Inpainting** | Image Engine | Credit | Alta | P1 |
| 9 | **Image-to-Video** | Video Engine | Credit | Muy Alta | P2 |
| 10 | **Quick Sprites (Video Gen → Sprite)** | Video Engine | Credit | Muy Alta | P2 |
| 11 | **SFX Generation** | Audio Engine | Credit | Alta | P1 |
| 12 | **Music Generation** | Audio Engine | Credit | Alta | P1 |
| 13 | **Speech / Voice Generation** | Audio Engine | Credit | Alta | P1 |
| 14 | **Tile Generator** | Tile Forge | Credit | Alta | P1 |
| 15 | **Seamless Texture Generator** | Tile Forge | Credit | Alta | P1 |
| 16 | **PBR Material Generator** | Material Forge | Credit | Muy Alta | P2 |
| 17 | **AI Coding Agent** | Agent Engine | Pro | Muy Alta | P1 |
| 18 | **Project Manager** | Asset Management | Free | Baja | P0 |
| 19 | **Asset Library / Browser** | Asset Management | Free | Media | P0 |
| 20 | **Model Configuration** | Model Config | Free | Media | P0 |
| 21 | **Game Publishing / Arcade** | Publishing | Pro | Alta | P3 |

### Descripciones detalladas

**1. Auto-Sprite**: Convierte un video (MP4, GIF, WEBM) en un spritesheet optimizado con metafile para motores de juego. El proceso incluye extracción de frames a FPS configurable, deduplicación perceptual de frames similares, packing óptimo en atlas, y generación de metadatos en formato Aseprite, Godot (.tres), Unity (.json) o TexturePacker. Soporta detección automática de regiones de sprite con fondo transparente.

**2. Sprite Sheet Slicer**: Herramienta para cortar spritesheets existentes en sprites individuales. Permite slicing manual por grid (filas/columnas, padding, offsets) o detección automática de sprites por bounding box de contenido. Exporta frames individuales o reorganiza el atlas con mejor compactación. Esencial para trabajar con assets comprados o descargados.

**3. Pixel Art Converter**: Transforma cualquier imagen de alta resolución en pixel art estilizado. El pipeline aplica downscale con nearest-neighbor, cuantización de color (algoritmo Wu o k-means), dithering seleccionable (Floyd-Steinberg, Bayer, Atkinson), realce de bordes opcional (Sobel), y generación de paleta exportable. También soporta aplicar una paleta predefinida (Pico-8, GameBoy, NES) para coherencia visual entre assets.

**4. 3D to 2D Renderer**: Renderiza modelos 3D (GLTF/GLB, OBJ) desde múltiples ángulos para generar sprites 2D isométricos o top-down. Configura cámara ortográfica o perspectiva, iluminación, y exporta frames de animación si el modelo tiene rig. Reduce drásticamente el tiempo de producción de sprite sets para juegos con perspectiva isométrica.

**5. Image Generation**: Generación de imágenes de game art desde prompt de texto. Soporta múltiples estilos presets (pixel art, cartoon, painterly, sci-fi, fantasy), ajuste de resolución, batch de variantes con distintos seeds, y selección del modelo a usar (local o cloud). El prompt se enriquece automáticamente con keywords de game art según el estilo seleccionado.

**6. Background Removal**: Eliminación del fondo de imágenes de forma automática usando modelos de segmentación semántica. Produce una imagen RGBA con canal alpha preciso alrededor del objeto principal. Soporta refino manual de la máscara con herramienta de pincel. Funciona especialmente bien en sprites de personajes y objetos sobre fondo uniforme, y también en imágenes complejas con fondos naturales.

**7. Outpainting / Image Expansion**: Extiende el canvas de una imagen más allá de sus bordes originales generando contenido coherente con el contexto visual. Permite expandir en cualquier dirección y cualquier cantidad de píxeles. Útil para crear fondos panorámicos a partir de una imagen base o para ajustar proporciones de un asset sin rehacerlo desde cero.

**8. Inpainting**: Rellena o edita regiones específicas de una imagen indicadas por una máscara. El modelo genera contenido coherente con el resto de la imagen. Casos de uso incluyen eliminar objetos, cambiar texturas de partes específicas, o corregir artefactos en imágenes generadas.

**9. Image-to-Video**: Genera un clip de video animado a partir de una imagen estática, con control de movimiento y estilo visual. Los clips son cortos (2-8 segundos) y están diseñados para ser usados como base para extraer spritesheets animados. Actualmente solo viable via APIs cloud (Kling, Wan, Veo) dado el VRAM requerido por los modelos locales.

**10. Quick Sprites**: Pipeline combinado que ejecuta Image-to-Video seguido de Auto-Sprite en un solo paso. El usuario define el personaje o elemento (via prompt o imagen), el número de frames y el FPS deseado, y obtiene directamente un spritesheet listo para usar. Abstrae completamente la complejidad de los pasos intermedios.

**11. SFX Generation**: Genera efectos de sonido para juegos desde una descripción textual (ej: "footstep on gravel", "sword clash metal", "magic spell cast"). Produce archivos WAV/OGG de corta duración (0.5-5 segundos). Soporta presets de categorías comunes (pasos, impactos, magia, interfaz, ambiental) y ajuste de parámetros (duración, variación aleatoria, seed).

**12. Music Generation**: Genera música de fondo para juegos desde una descripción de estilo y mood (ej: "upbeat adventure loop, 120 BPM, chiptune"). Produce loops de 15-60 segundos diseñados para reproducción continua sin cortes perceptibles. La mayor limitación técnica actual es la ausencia de modelos open source con licencia comercial libre que igualen la calidad de Suno/Udio.

**13. Speech / Voice Generation**: Síntesis de voz para diálogos de personajes, narración o tutoriales. Soporta múltiples perfiles de voz, ajuste de velocidad y entonación, y clonación de voz a partir de una muestra de referencia de 10-15 segundos. Produce archivos WAV/OGG listos para integrar en el motor de juego.

**14. Tile Generator**: Genera tiles individuales desde prompt de texto, con control del estilo visual, bioma o ambiente (bosque, mazmorra, cielo), y categoría (suelo, muro, decoración). Los tiles se generan garantizando que sean seamless por defecto. Permite generar variantes de un tile base (dañado, mojado, cubierto de nieve) con coherencia visual.

**15. Seamless Texture Generator**: Genera texturas tileable desde prompt o imagen de referencia. Aplica técnicas de mirror padding, ControlNet tile y verificación de tileabilidad automática. Soporta texturas PBR simples (solo albedo) o completas (albedo + normal + roughness). También puede hacer seamless una textura existente que no lo sea.

**16. PBR Material Generator**: Genera sets completos de mapas PBR (albedo, normal, roughness, metallic, ambient occlusion, height) desde un prompt de texto o imagen de referencia. Los mapas son coherentes entre sí y listos para usar en Godot, Unity o Unreal. El pipeline infiere los mapas derivados (normal desde depth, roughness desde luminance del albedo) cuando no se generan directamente.

**17. AI Coding Agent**: Agente de IA conversacional que genera, edita y refactoriza código para motores de juego (Godot GDScript, Unity C#, HTML5/Phaser). El agente mantiene el contexto del workspace, genera un plan de pasos visible al usuario, ejecuta los cambios en un sandbox, muestra diffs coloreados de cada modificación, crea checkpoints para rollback, y puede delegar a otros engines (ej: pedir al Image Engine que genere una textura que el código necesita).

**18. Project Manager**: Gestión del ciclo de vida de proyectos: crear, abrir, renombrar, archivar. Cada proyecto tiene un directorio raíz en el filesystem del usuario donde todos los assets se organizan. Soporta múltiples proyectos abiertos simultáneamente en diferentes ventanas.

**19. Asset Library / Browser**: Navegador de todos los assets del proyecto con filtrado por tipo, tags, fecha y origen (importado, generado, derivado). Muestra previews en miniatura, metadata completa, y el árbol de linaje (de qué asset se derivó este). Permite organizar assets en colecciones y exportar selecciones.

**20. Model Configuration**: Panel de configuración para gestionar los modelos de IA disponibles: registrar API keys (almacenadas en el keychain del OS), configurar rutas de modelos locales, definir reglas de routing (qué modelo usar para qué operación y calidad), y ver el uso de quota por proveedor.

**21. Game Publishing / Arcade**: Empaqueta el juego en un formato jugable y lo publica en plataformas de distribución (itch.io, GameJolt) o genera un embed web. Incluye una vista de arcade donde el usuario puede jugar su juego directamente dentro de Sorceress. Es el context más downstream y el de menor prioridad en el MVP.

---

## 3. Modelos de IA — Estado del Arte (Abril 2026)

Esta sección sintetiza las opciones disponibles por categoría de feature, con análisis y veredicto para el MVP. El criterio principal es: **licencia Apache 2.0 o MIT primero**, luego calidad, luego viabilidad en hardware del usuario promedio (16-24 GB VRAM).

### 3.1 Image Generation

| Tipo | Nombre | Params | VRAM | Licencia | Calidad |
|------|--------|--------|------|----------|---------|
| Open source | FLUX.1-schnell | 12B | 12 GB (8 GB Q4) | Apache 2.0 | 4/5 |
| Open source | FLUX.1-dev | 12B | 16 GB | No comercial | 4.5/5 |
| Open source | SD 3.5 Large | 8B | 16 GB | Stability (libre <$1M) | 4/5 |
| Open source | SD 3.5 Medium | 2.5B | 8 GB | Stability | 3.5/5 |
| API | fal.ai FLUX schnell | — | — | — (API) | 4/5 |
| API | BFL FLUX1.1 pro Ultra | — | — | — (API) | 5/5 |
| API | FLUX.2 [pro] (Black Forest Labs) | — | — | — (API) | 4.5/5 ($0.03/img) |
| API | FLUX.2 [max] (Black Forest Labs) | — | — | — (API) | 5/5 ($0.07/img) |
| Clasico | Perlin noise + L-systems | — | — | — | N/A |

**Libreria Rust**: `candle` (HuggingFace) para inferencia local via safetensors/GGUF; `ort` (ONNX Runtime) para modelos exportados; `reqwest` + `serde_json` para APIs.

**Nota (nov 2025 – ene 2026)**: FLUX.2 [pro] ($0.03/img) y FLUX.2 [max] ($0.07/img) son los modelos top de pago de Black Forest Labs. FLUX.1 Kontext introduce un nuevo paradigma de edicion in-context (inpainting guiado por instruccion de lenguaje natural) especialmente util para edicion iterativa de sprites.

**Veredicto MVP**: FLUX.1-schnell como modelo local principal. Es la unica opcion que combina licencia Apache 2.0, calidad ≥4/5 y convergencia en 1-4 pasos (vs 20-50 pasos de SD 1.x). En hardware con <12 GB VRAM, usar version Q4 cuantizada (~8 GB) o hacer fallback a FLUX.2 [pro] via API ($0.03/imagen). No usar FLUX.1-dev en produccion: su licencia no-comercial crea riesgo legal.

### 3.2 Background Removal

| Tipo | Nombre | Params | VRAM | Licencia | Calidad |
|------|--------|--------|------|----------|---------|
| Open source | BiRefNet | ~100M | 4 GB | MIT | 4.5/5 |
| Open source | BiRefNet-lite | ~50M | 2 GB | MIT | 4/5 |
| Open source | SAM 2.1 Large | ~310M | 4 GB | Apache 2.0 | 5/5 |
| Open source | SAM 2.1 Base+ | ~80M | 2 GB | Apache 2.0 | 4.5/5 |
| Open source | RMBG-2.0 | ~200M | 2 GB | CC BY-NC | 5/5 |
| API | fal.ai RMBG-2.0 | — | — | — | 5/5 |
| API | fal.ai Background Removal | — | — | ~$0.003/img | 4.5/5 |
| Clasico | GrabCut (OpenCV) | — | CPU | — | 3/5 |

**Libreria Rust**: `ort` con modelo BiRefNet exportado a ONNX; `imageproc` para post-proceso de mascaras (morfologia, suavizado de bordes).

**Veredicto MVP**: BiRefNet (MIT) para batch processing automatico sin interaccion del usuario; SAM 2.1 Base+ para segmentacion interactiva donde el usuario indica puntos de referencia. RMBG-2.0 tiene mejor calidad pero su licencia CC BY-NC es un problema para uso comercial sin acuerdo con BRIA AI. No bloquear el MVP por esto: BiRefNet es suficiente.

**Analisis critico**: SAM 2.1 es superiormente flexible (puede segmentar cualquier objeto con prompt de punto/caja), pero BiRefNet esta optimizado especificamente para separacion objeto/fondo, que es el caso de uso dominante en game dev. Usar ambos segun el contexto es la estrategia correcta.

### 3.3 Pixel Art Conversion

No existe un modelo de IA con licencia libre que supere consistentemente a los algoritmos clasicos para esta tarea. El pipeline optimo es puramente clasico:

| Paso | Algoritmo | Libreria Rust | Complejidad |
|------|-----------|---------------|-------------|
| Downscale | Nearest-neighbor | `image` (FilterType::Nearest) | O(1) |
| Cuantizacion de color | Wu's Algorithm o k-means | `quantette` | O(n) |
| Dithering | Floyd-Steinberg o Bayer | implementacion custom | O(n) |
| Outline | Sobel + thresholding | `imageproc` | O(n) |
| Paleta exportable | k-means en espacio Lab | `palette` | O(n·k·i) |

**Veredicto MVP**: Implementacion Rust pura con `image` + `imageproc` + `quantette`. Sin dependencias de modelos IA. Calidad suficiente para el 95% de los casos de uso. El crate `quantette` implementa Wu's algorithm que es O(n) y produce paletas perceptualmente optimas. Para casos especiales con LoRA de pixel art (SDXL), delegar al Image Engine con model profile "pixel-art-xl".

### 3.4 SFX Generation

| Tipo | Nombre | Tamaño | VRAM | Licencia | Calidad |
|------|--------|--------|------|----------|---------|
| Open source | AudioLDM 2 | ~2.5 GB | 8 GB | Apache 2.0 | 3.5/5 |
| Open source | AudioLDM 2-Full | ~3.5 GB | 10 GB | Apache 2.0 | 4/5 |
| Open source | Stable Audio Open 1.0 | ~2 GB | 8 GB | Stability | 4/5 |
| Open source | Tango 2 | ~2 GB | 8 GB | Apache 2.0 | 3.5/5 |
| API | ElevenLabs SFX (suite unificada) | — | — | desde $6/mes Starter | 4.5/5 |
| API | Stability AI Audio | — | — | $0.012/seg | 4/5 |
| Clasico | Sintesis FM + ADSR | — | CPU | — | Retro |

**Libreria Rust**: `cpal` (playback cross-platform), `rodio` (alto nivel sobre cpal), `hound` (WAV I/O), `symphonia` (decode multi-formato).

**Veredicto MVP**: AudioLDM 2 con licencia Apache 2.0 para inferencia local. Genera hasta 10 segundos de SFX desde texto descriptivo. El modelo entiende bien descriptores de game audio ("footsteps on stone", "laser beam", "coin pickup chime"). Para hardware sin GPU capaz, fallback a ElevenLabs SFX API (plan Starter $6/mes incluye SFX + Music + TTS en una sola integracion).

**Nota critica**: AudioGen Medium de Meta tiene mejor calidad pero su licencia CC BY-NC lo excluye de uso comercial. Stable Audio Open 1.0 tiene mejor calidad que AudioLDM 2 pero la licencia Stability AI tiene restricciones para revenue >$1M. Para el MVP (volumen bajo), Stable Audio Open es una opcion valida. **ElevenLabs consolida desde 2025 una suite completa (SFX + Music + TTS + Voice Cloning) accesible desde $6/mes Starter**, lo que simplifica la integracion al usar un unico proveedor para todo el audio cloud.

### 3.5 Music Generation

Esta categoria tiene la brecha mas grande entre open source y closed source. No existe a abril 2026 un modelo de musica open source con licencia Apache 2.0/MIT que iguale la calidad de Suno v4 o Udio.

| Tipo | Nombre | Licencia | Calidad | Limitacion |
|------|--------|----------|---------|------------|
| Open source | MusicGen Large | CC BY-NC | 4/5 | NO comercial |
| Open source | Stable Audio Open | Stability | 3.5/5 | Loops cortos |
| API | Suno v4 | — | 5/5 | Basic $8/mes (500 canciones) |
| API | Udio | — | 4.5/5 | $10/mes |
| API | ElevenLabs Music (suite) | — | 4/5 | Incluido desde $6/mes Starter |
| Clasico | MIDI procedural + Markov | MIT | Variable | Requiere diseno manual |

**Veredicto MVP**: Dos estrategias en paralelo. (1) Para usuarios que priorizan calidad: Suno API (Basic $8/mes, 500 canciones) o ElevenLabs Music (integrada en la misma suite que SFX y TTS, desde $6/mes) con costos transparentes. (2) Para usuarios que priorizan costo cero: sintesis procedural MIDI con samples de Game Music Kit (dominio publico/CC0) usando `midly` + `fundsp`. La segunda opcion produce musica de menor calidad pero completamente libre. **No intentar usar MusicGen en produccion comercial sin acuerdo explico de licencia.**

### 3.6 Speech / Voice Generation

| Tipo | Nombre | Tamaño | VRAM | Licencia | Calidad |
|------|--------|--------|------|----------|---------|
| Open source | Kokoro-82M | 330 MB | CPU ok | Apache 2.0 | 4/5 |
| Open source | Parler-TTS Mini | 880M | 4 GB | Apache 2.0 | 3.5/5 |
| Open source | F5-TTS | ~300 MB | 2 GB | CC BY-NC | 4/5 |
| Open source | XTTS v2 | ~1.8 GB | 4 GB | Coqui NC | 4.5/5 |
| API | ElevenLabs (suite unificada) | — | — | desde Starter $6/mes (licencia comercial) | 5/5 |
| API | OpenAI gpt-realtime-1.5 | — | — | $0.06/min | 4.5/5 |
| API | Cartesia | — | — | $0.015/min | 4.5/5 |

**Libreria Rust**: `ort` con Kokoro exportado a ONNX (ya hay exportaciones disponibles en HuggingFace); `rubato` para resampling; `hound` para output WAV.

**Veredicto MVP**: Kokoro-82M (Apache 2.0) para TTS estandar. A pesar de sus solo 82 millones de parametros, produce voz de calidad sorprendente en ingles con latencia de 50-100ms en CPU moderna. No requiere GPU. Para voice cloning (caracteristica avanzada), usar F5-TTS via API externa dado que su licencia CC BY-NC lo excluye del producto comercial directamente. ElevenLabs para maxima calidad en el tier Pro (suite completa SFX + Music + TTS desde $6/mes Starter). Para NPCs con dialogo en tiempo real, OpenAI gpt-realtime-1.5 ofrece la menor latencia de respuesta de voz conversacional.

### 3.7 Image-to-Video

Esta categoria es la mas costosa en VRAM. Los modelos locales requieren 16-24 GB de VRAM, lo que excluye la mayoria del hardware consumer.

| Tipo | Nombre | Params | VRAM | Licencia | Calidad |
|------|--------|--------|------|----------|---------|
| Open source | CogVideoX-5B | 5B | 16 GB | Apache 2.0 | 4/5 |
| Open source | Wan2.1 14B | 14B | 24 GB | Apache 2.0 | 4.5/5 |
| Open source | AnimateDiff v3 | ~3 GB | 8 GB | Apache 2.0 | 3/5 |
| API | fal.ai Kling 2.5 | — | — | $0.07/seg | 4.5/5 |
| API | fal.ai Wan 2.5 | — | — | $0.05/seg | 4/5 |
| API | fal.ai Veo 3 | — | — | $0.40/seg | 5/5 |

**Veredicto MVP**: Delegar completamente a APIs cloud en el MVP. AnimateDiff sobre SD es la unica opcion en 8 GB VRAM pero su calidad (3/5) es insuficiente para el caso de uso principal. fal.ai + Wan 2.5 a $0.05/segundo es la mejor relacion calidad/precio para el tier Pro. Implementar inferencia local de CogVideoX-5B como feature opt-in para usuarios con RTX 3090/4090 (16+ GB VRAM).

### 3.8 PBR Materials

No existe un modelo open source maduro y con licencia libre para generar sets PBR completos desde texto. El pipeline mas practico combina generacion de albedo + derivacion computacional de los mapas restantes:

| Paso | Metodo | Herramienta Rust | Calidad |
|------|--------|------------------|---------|
| Albedo | FLUX.1-schnell (Apache 2.0) | candle | 4/5 |
| Depth | Depth-Anything-V2 (Apache 2.0) via ONNX | ort | 4/5 |
| Normal desde Depth | Gradiente Sobel 3D (algoritmo clasico) | imageproc + glam | 3.5/5 |
| Roughness | Heuristica: inversa de luminance del albedo | custom | 3/5 |
| Metallic | Clasificacion por saturacion/valor HSV | custom | 3/5 |
| AO | SSAO en shader | wgpu | 3.5/5 |

**APIs 3D especializadas (2025-2026)**:

| API | Modelo | Precio aprox. | Calidad | Nota |
|-----|--------|---------------|---------|------|
| Meshy | Meshy 5 | por creditos | 4.5/5 | Plugin oficial para Godot; assets 3D + PBR desde texto/imagen |
| Tripo | Tripo v3.0 Ultra | por creditos | 4.5/5 | Alta fidelidad 3D con mapas PBR completos |
| Stability AI | — | API | 4/5 | Texturizado PBR desde imagen |

**Veredicto MVP**: Pipeline hibrido clasico-IA para uso rapido. La calidad de los mapas derivados (normal, roughness) sera inferior a herramientas especializadas como Adobe Substance 3D, pero es funcional para prototipos y juegos indie donde la fidelidad PBR exacta no es critica. Para usuarios que necesitan calidad profesional, Meshy 5 (con su plugin Godot) es la opcion API principal; Tripo v3.0 Ultra como alternativa de alta fidelidad.

### 3.9 AI Coding Agent

| Tipo | Nombre | Params | VRAM | Licencia | Calidad codigo |
|------|--------|--------|------|----------|----------------|
| Open source local | Qwen3 72B | 72B | 48 GB (BF16) / 40 GB (int4) | Apache 2.0 | 5/5 |
| Open source local | Qwen3 14B | 14B | 16 GB int4 | Apache 2.0 | 4.5/5 |
| Open source local | Llama 4 Maverick | ~24B activos | 24 GB | Llama 4 Community | 4.5/5 |
| Open source local | Llama 4 Scout | ~16B activos | 16 GB | Llama 4 Community | 4/5 |
| Open source local | Qwen2.5-Coder-32B | 32B | 24 GB (BF16) / 16 GB (int4) | Apache 2.0 | 4.5/5 (superado por Qwen3) |
| Open source local | Qwen2.5-Coder-7B | 7B | 6 GB int4 | Apache 2.0 | 4/5 |
| Open source local | DeepSeek-V3 | 671B MoE | Cluster | Si | 5/5 |
| API | Claude Sonnet 4.6 | — | — | $3/$15 per 1M | 5/5 |
| API | GPT-5.4 | — | — | $2.50/$15 per 1M | 5/5 |
| API | DeepSeek-V3.2 API | — | — | $0.28/$0.42 per 1M | 5/5 |
| API | Qwen2.5-Coder-32B (Together AI) | — | — | $0.80/$0.80 per 1M | 4.5/5 |

**Libreria Rust**: `llama-cpp-rs` o proceso sidecar llama.cpp para modelos GGUF locales; `async-openai` para APIs OpenAI-compatible; `reqwest` para Anthropic API; `tree-sitter` para analisis AST del codigo generado.

**Nota sobre modelos OSS actualizados**: Qwen3 (14B y 72B) supera a Qwen2.5-Coder en calidad de codigo y razonamiento. Qwen3 14B (~16 GB VRAM int4) es la nueva recomendacion para el tier gratuito local. DeepSeek-V3.2 incluye modo thinking integrado y es ~10x mas barato que Claude manteniendo calidad comparable para codigo.

**Veredicto MVP**: Estrategia de dos niveles. Nivel 1 (gratuito): Qwen3 14B local via llama.cpp GGUF Q4 (~10 GB RAM, sin GPU) — calidad superior a la generacion anterior para tareas de coding. Nivel 2 (Pro): Claude Sonnet 4.6 via API — maxima calidad, capaz de generar sistemas completos con razonamiento profundo. DeepSeek-V3.2 API es una alternativa economica a Claude ($0.28 input vs $3 de Anthropic) con calidad comparable o superior para codigo.

### Tabla resumen de decisiones para MVP

| Categoria | Modelo local (MVP) | API fallback | Licencia local |
|-----------|-------------------|--------------|----------------|
| Image Generation | FLUX.1-schnell Q4 | FLUX.2 [pro] $0.03/img | Apache 2.0 |
| Background Removal | BiRefNet | fal.ai $0.003/img | MIT |
| Pixel Art | Algoritmos clasicos | — | — |
| SFX | AudioLDM 2 | ElevenLabs (suite, $6/mes) | Apache 2.0 |
| Music | MIDI procedural | Suno v4 ($8/mes) o ElevenLabs Music | — |
| Speech | Kokoro-82M | ElevenLabs (suite) | Apache 2.0 |
| Image-to-Video | — (solo cloud MVP) | fal.ai Wan 2.5 | — |
| PBR Materials | Pipeline hibrido | Meshy 5 / Stability AI API | Apache 2.0 |
| Coding Agent | Qwen3 14B GGUF | Claude Sonnet 4.6 | Apache 2.0 |

---

## 4. Arquitectura del Sistema

### 4.1 Vision General

La arquitectura separa el sistema en dos planos con responsabilidades claras y comunicacion asincrona entre ellos:

```
╔══════════════════════════════════════════════════════════════════════╗
║                         CONTROL PLANE                               ║
║                  (proceso Tauri principal)                           ║
║                                                                      ║
║  ┌─────────────────────────────────────────────────────────────┐     ║
║  │  Frontend WebView (React/SvelteKit)                          │     ║
║  │  • UI components (shadcn/ui, Tailwind)                       │     ║
║  │  • Canvas editor (PixiJS + Konva)                            │     ║
║  │  • State management (Zustand / Svelte Stores)                │     ║
║  │  • Tauri IPC wrappers                                        │     ║
║  └──────────────────────┬──────────────────────────────────────┘     ║
║                         │ Tauri IPC (commands / events / channels)   ║
║  ┌──────────────────────▼──────────────────────────────────────┐     ║
║  │  Rust Backend (Tauri process)                                │     ║
║  │  ┌────────────┐  ┌────────────┐  ┌────────────┐             │     ║
║  │  │  Asset     │  │  Job Queue │  │  Model     │             │     ║
║  │  │  Mgmt      │  │  Context   │  │  Config    │             │     ║
║  │  └────────────┘  └─────┬──────┘  └────────────┘             │     ║
║  │  ┌────────────┐        │                                     │     ║
║  │  │ Identity & │        │ dispatch via IPC socket             │     ║
║  │  │ Prefs      │        │                                     │     ║
║  │  └────────────┘        │                                     │     ║
║  │  SQLite (sqlx) — assets, jobs, config, projects              │     ║
║  └─────────────────────────────────────────────────────────────┘     ║
╚══════════════════════════════════════════════════════════════════════╝
                           │
         ┌─────────────────┼──────────────────────────────┐
         │IPC socket       │IPC socket      IPC socket     │
         ▼                 ▼                               ▼
╔═════════════════╗ ╔══════════════════╗ ╔══════════════════════════╗
║  DATA PLANE     ║ ║  DATA PLANE      ║ ║  DATA PLANE              ║
║                 ║ ║                  ║ ║                          ║
║ sprite-worker   ║ ║  image-worker    ║ ║  audio-worker            ║
║ ─────────────── ║ ║  ──────────────  ║ ║  ──────────────────────  ║
║ • FFmpeg        ║ ║  • FLUX local    ║ ║  • AudioLDM 2            ║
║ • atlas packing ║ ║  • BiRefNet ONNX ║ ║  • Kokoro-82M            ║
║ • pixel art     ║ ║  • ComfyUI API   ║ ║  • rodio/cpal            ║
║ • crunch        ║ ║  • cloud APIs    ║ ║  • cloud APIs            ║
╚═════════════════╝ ╚══════════════════╝ ╚══════════════════════════╝
         │                 │                               │
╔═════════════════╗ ╔══════════════════╗ ╔══════════════════════════╗
║ tile-worker     ║ ║  material-worker ║ ║  agent-worker            ║
║ ─────────────── ║ ║  ──────────────  ║ ║  ──────────────────────  ║
║ • noise/WFC     ║ ║  • depth est.    ║ ║  • LLM streaming API     ║
║ • seamless      ║ ║  • normal maps   ║ ║  • llama.cpp sidecar     ║
║ • Wang tiles    ║ ║  • wgpu preview  ║ ║  • tree-sitter AST       ║
╚═════════════════╝ ╚══════════════════╝ ╚══════════════════════════╝

video-worker: CogVideoX / AnimateDiff / cloud APIs (fal.ai, Runway)
```

**Control Plane**: Baja latencia. Sin GPU. Sin bloqueos. Responde a la UI en <50ms. Solo orquesta: valida inputs, crea jobs, resuelve modelo, trackea progreso, persiste en SQLite, notifica al frontend via eventos Tauri.

**Data Plane**: Alta latencia. Puede usar GPU. Cada worker es un proceso OS independiente. No tiene estado propio — recibe jobs completos y devuelve resultados. El Control Plane detecta workers caidos y los relanza automaticamente.

### 4.2 Bounded Contexts (DDD)

Se identifican 11 bounded contexts organizados en tres planos funcionales:

| Context | Plano | Responsabilidad | Ubiquitous Language (terminos clave) |
|---------|-------|-----------------|--------------------------------------|
| Asset Management | Control | Ciclo de vida de todos los assets: guardar, versionar, organizar, exponer | Asset, AssetVersion, Collection, Project, DerivedFrom, ImportSource |
| Job Queue | Control | Recibir, priorizar, despachar y trackear jobs hacia workers | Job, JobSpec, JobPriority, WorkerCapacity, JobDependency, Retry |
| Model Configuration | Control | Gestionar modelos disponibles, API keys, routing, fallback chains | ModelProvider, ModelProfile, RoutingRule, FallbackChain, Quota |
| Identity & Prefs | Control | Perfil de usuario, preferencias UI, tier (Free/Pro), historial de uso | UserProfile, Preference, License, Tier |
| Sprite Factory | Data | Extraccion de frames, slicing, pixel art, atlas packing | SpriteSheet, Frame, AnimationClip, SliceGrid, PixelArtStyle, AtlasLayout |
| Image Engine | Data | Generacion, inpainting, outpainting, background removal | Prompt, Seed, DiffusionParams, ControlNet, Mask, Generation, Batch |
| Audio Engine | Data | SFX, musica, voz | AudioClip, SfxDescriptor, MusicGenre, VoiceLine, VoiceProfile, Loop |
| Video Engine | Data | Video generation, quick sprites | VideoClip, VideoPrompt, AnimationStyle, MotionGuidance, QuickSprite |
| Tile Forge | Data | Tilesets seamless, Wang tiles, auto-tiling | Tile, Tileset, SeamlessTile, WangTile, TileVariant, Biome, TileRule |
| Material Forge | Data | Mapas PBR completos desde imagen o prompt | PBRMaterial, AlbedoMap, NormalMap, RoughnessMap, MetallicMap, MaterialPrompt |
| Agent Engine | Data | Coding agent: planificacion, ejecucion, checkpoints | AgentSession, Task, Plan, Step, AgentThought, CodeChange, Checkpoint, Workspace |
| Publishing | Cross-cutting | Empaquetado y publicacion del juego | GameBuild, LayoutTemplate, ArcadeEntry, PublishTarget, PlaySession |

**Shared Kernel** (tipos compartidos entre todos los contexts, en crate `sorceress-shared-kernel`):

```rust
pub struct AssetId(Uuid);
pub struct JobId(Uuid);
pub struct ProjectId(Uuid);
pub struct SessionId(Uuid);
pub struct StoragePath(PathBuf);   // siempre relativo al project root_path
pub struct Timestamp(DateTime<Utc>);
pub struct Checksum(blake3::Hash);

pub enum ErrorKind {
    NotFound, ValidationError, ProcessingError,
    ExternalApiError, InsufficientResources, Cancelled,
}

pub trait DomainEvent: Send + Sync {
    fn event_id(&self) -> Uuid;
    fn occurred_at(&self) -> Timestamp;
    fn aggregate_id(&self) -> String;
    fn event_type(&self) -> &'static str;
}
```

**Regla de oro del Shared Kernel**: Un tipo solo va al Shared Kernel si cruza la frontera de al menos 3 contexts frecuentemente. Si un tipo solo se usa en un context, vive en ese context.

### 4.3 Context Map (relaciones entre contexts)

```
                 ┌──────────────────────────────────┐
                 │         SHARED KERNEL             │
                 │  AssetId, JobId, StoragePath,     │
                 │  Timestamp, Checksum, ErrorKind   │
                 └──────────────────────────────────┘
                              │ usa todos
         ┌────────────────────┼──────────────────────┐
         │                   │                       │
  ┌──────▼──────┐      ┌──────▼──────┐        ┌──────▼──────┐
  │ Identity &  │      │ Model Config│        │ Asset Mgmt  │
  │ Prefs       │      │             │        │             │
  │ (Upstream   │      │ (Upstream   │        │ (integra    │
  │ Published   │      │ Customer/   │        │ todos los   │
  │ Language)   │      │ Supplier)   │        │ outputs)    │
  └──────┬──────┘      └──────┬──────┘        └──────┬──────┘
         │ informa            │ router               │ AssetRegistered
         ▼                    ▼                       ▼
  ┌──────────────────────────────────────────────────────────┐
  │              JOB QUEUE CONTEXT                            │
  │         (Customer/Supplier con todos los engines)         │
  └──┬──────────┬──────────┬──────────┬──────────┬───────────┘
     │dispatch  │dispatch  │dispatch  │dispatch  │dispatch
     ▼          ▼          ▼          ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│Sprite  │ │Image   │ │Audio   │ │Video   │ │Tile    │ │Material│
│Factory │ │Engine  │ │Engine  │ │Engine  │ │Forge   │ │Forge   │
└───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘ └───┬────┘
    └──────────┴──────────┴──────────┴──────────┴────────────┘
                        │ XxxJobCompleted events
                        ▼
              ┌──────────────────┐
              │  ASSET MANAGEMENT│
              └────────┬─────────┘
                       ▼ AssetRegistered
               ┌───────────────┐
               │  PUBLISHING   │
               └───────────────┘

┌─────────────────────────────────────────────┐
│  AGENT ENGINE (usa Job Queue + Asset Mgmt + │
│  Model Config via ACL)                      │
└─────────────────────────────────────────────┘
```

| Upstream | Downstream | Patron | Descripcion |
|----------|------------|--------|-------------|
| Identity & Prefs | Todos | Published Language | Expone UserProfile y Tier; todos conforman |
| Model Config | Job Queue | Customer/Supplier | Job Queue consulta que modelo usar al despachar |
| Model Config | Todos los Engines | Customer/Supplier | Cada engine recibe ModelProfile ya resuelto en el JobSpec |
| Job Queue | Todos los Engines | Customer/Supplier | Job Queue despacha; cada engine es supplier de procesamiento |
| Todos los Engines | Asset Management | Conformist | Los engines publican eventos de completitud; Asset Management registra outputs |
| Asset Management | Publishing | Customer/Supplier | Publishing solo publica assets ya registrados |
| Agent Engine | Model Config | ACL | Agent Engine tiene su propio modelo de "LLM" que traduce via ACL al ModelProfile |
| Agent Engine | Job Queue | Customer | Agent Engine puede enviar jobs de Image/Audio como parte de sus steps |

### 4.4 Control Plane: Bounded Contexts y Responsabilidades

**Contexts en el Control Plane**: Asset Management, Job Queue, Model Configuration, Identity & Prefs, Publishing (parte de orquestacion).

**Responsabilidades del Control Plane**:
- Recibir acciones de la UI via Tauri IPC (commands sincronos)
- Validar inputs y resolver el modelo apropiado via Model Config
- Crear Jobs con prioridad y dependencias via Job Queue
- Trackear progreso de jobs y notificar frontend via Tauri Events
- Mantener el estado del proyecto en SQLite (assets, colecciones, versiones)
- Detectar workers caidos via heartbeat timeout y relanzarlos
- Enrutar resultados: registrar outputs como Assets y notificar al frontend

**Puertos y adaptadores del Control Plane**:

```
Inbound ports:
  Tauri IPC Adapter       -- comandos desde la WebView (invoke)
  Worker RPC Adapter      -- heartbeats y resultados desde workers via IPC socket
  HTTP Webhook Adapter    -- callbacks de APIs cloud (Replicate, etc.)

Outbound ports:
  SqliteJobRepository     -- persistencia de jobs, assets, proyectos
  FilesystemAdapter       -- validacion de paths, lectura de metadata
  WorkerIPCDispatcher     -- despacho de jobs a workers via Unix socket / Named Pipe
  TokioBroadcastPublisher -- event bus interno para eventos de dominio
  TauriEventEmitter       -- notificacion al frontend via tauri::emit
  OSKeychainAdapter       -- almacenamiento seguro de API keys
```

**Stack del Control Plane**: Proceso Tauri (tokio async runtime), SQLite via sqlx, Tauri commands/events/channels para IPC con frontend, Unix Domain Sockets o Named Pipes para comunicacion con workers.

### 4.5 Data Plane: Workers de Procesamiento

**Contexts en el Data Plane**: Sprite Factory, Image Engine, Audio Engine, Video Engine, Tile Forge, Material Forge, Agent Engine.

**Cada worker es un proceso OS independiente** lanzado como sidecar de Tauri:

```
sorceress (proceso Tauri principal)
  │
  ├── sprite-worker    (FFmpeg, image processing, atlas packing)
  ├── image-worker     (FLUX/SDXL local, ComfyUI API, BiRefNet, cloud APIs)
  ├── audio-worker     (AudioLDM 2, Kokoro, rodio, cloud APIs)
  ├── video-worker     (CogVideoX, AnimateDiff, cloud APIs)
  ├── tile-worker      (noise, WFC, seamless algorithms)
  ├── material-worker  (depth estimation, normal maps, wgpu preview)
  └── agent-worker     (LLM streaming, llama.cpp, tree-sitter, sandbox)
```

**Ciclo de vida de un worker**:
1. Tauri app lanza el worker como subprocess al iniciar o en demanda
2. Worker abre socket IPC, envia `WorkerRegistered` con su `WorkerKind` y capacidad de VRAM
3. Worker entra en loop: espera mensajes `dispatch_job` para su `WorkerKind`
4. Al recibir un job: procesa, reporta progreso cada ~1 segundo, envia `job_completed` o `job_failed`
5. Control Plane detecta ausencia de heartbeat (timeout configurable, default 30s) → `WorkerDied` → relanza

**Prioridades de la job queue**:

```
Critical  = 100  -- jobs bloqueantes de UI: preview en vivo, analisis rapido
High      = 75   -- jobs iniciados directamente por el usuario
Normal    = 50   -- generacion estandar
Background = 25  -- batch processing, pre-generacion, tareas de mantenimiento
```

**GPU slot management**: Los workers reportan su VRAM disponible en cada heartbeat. El Job Queue no despacha un job a un worker si no hay VRAM suficiente, manteniendo el job en estado `Queued` hasta que el worker libere recursos.

### 4.5 Comunicacion entre planos

**Protocolo IPC**: JSON sobre Unix Domain Sockets (macOS/Linux) y Named Pipes (Windows). Framing: prefijo de 4 bytes big-endian con la longitud del mensaje + JSON body. La simplicidad de JSON se prefiere sobre MessagePack en el MVP ya que los jobs son de larga duracion y el overhead de serializacion es irrelevante.

**Mensajes del protocolo**:

```
Control → Worker (dispatch):
{ "msg_type": "dispatch_job",
  "job_id": "uuid",
  "operation": "autosprite.generate",
  "params": { ... },
  "model_profile": { ... },
  "deadline_ms": 300000 }

Worker → Control (progreso):
{ "msg_type": "job_progress",
  "job_id": "uuid",
  "progress": 0.45,
  "message": "Extracting frames: 22/48" }

Worker → Control (completado):
{ "msg_type": "job_completed",
  "job_id": "uuid",
  "output_paths": ["path/a.png", "path/a.tres"],
  "duration_ms": 4230,
  "metadata": { "frame_count": 48 } }

Worker → Control (heartbeat):
{ "msg_type": "heartbeat",
  "worker_id": "uuid",
  "worker_kind": "SpriteFactory",
  "capacity": { "cpu_slots_free": 2, "vram_mb_free": 4096 } }
```

**Alternativa para modelos pequenos**: Workers con modelos <2 GB de VRAM pueden embeberse directamente en el proceso Tauri usando `candle` (sin sidecar), eliminando el overhead de IPC. Esto aplica a BiRefNet, Kokoro-82M y modelos ONNX pequenos.

---

## 5. Stack Tecnologico Detallado

### 5.1 Frontend

**Framework recomendado**: **SvelteKit** con adapter-static y Vite. Alternativamente React + Vite si el equipo ya lo conoce.

| Criterio | SvelteKit | React | Razon de peso |
|----------|-----------|-------|----------------|
| Bundle size | ~20 KB runtime | ~130 KB | En desktop la diferencia es marginal pero Svelte es mas limpio |
| Reactividad | Signals nativos (sin Virtual DOM) | Virtual DOM | Svelte evita re-renders innecesarios sin boilerplate |
| Ecosistema UI | Melt UI, Bits UI | shadcn/ui, Radix | shadcn/ui es superior en componentes de calidad |
| Curva de aprendizaje | Baja | Media | Svelte es mas accesible para desarrolladores nuevos |
| Veredicto MVP | **Preferida** | Valida si hay experiencia | La diferencia no es critica |

**Librerias UI**:
- `shadcn/ui` (React): componentes copiables sobre Radix UI + Tailwind CSS. Dark mode, accesibilidad ARIA, sin vendor lock-in.
- `Melt UI` o `Bits UI` (Svelte): equivalente de shadcn/ui para Svelte.
- `Tailwind CSS 4.x`: sistema de utilidades, paleta personalizada para el dark theme de game dev tools.

**Canvas y rendering**:

| Libreria | Renderer | Caso de uso en Sorceress |
|----------|----------|--------------------------|
| PixiJS 8.x | WebGL/WebGPU | Sprite viewer, animation playback, tilemap preview |
| Konva.js | Canvas2D | Editor interactivo: seleccion, drag, resize, layers |
| Fabric.js | Canvas2D | Composicion de assets, edicion vectorial ligera |
| Three.js | WebGL | Preview 3D para Material Forge y 3D-to-2D renderer |

**State management**:
- Zustand (React): store centralizado sin boilerplate excesivo; `immer` para mutaciones inmutables.
- Svelte Stores: reactividad nativa con `writable`/`derived`/`readable`; no se necesita libreria externa.

**Paleta dark theme** (para `tailwind.config`):
```
canvas:  #1a1a2e  -- fondo principal del area de trabajo
panel:   #16213e  -- fondos de paneles laterales
surface: #0f3460  -- cards, dialogs
accent:  #e94560  -- botones primarios, highlights
text:    #a8b2d8  -- texto secundario
```

### 5.2 Backend Rust

**Estructura del workspace Cargo**:

```
sorceress/
├── Cargo.toml                     -- workspace root
├── package.json                   -- frontend tooling
├── vite.config.ts
│
├── src/                           -- Frontend (SvelteKit o React)
│   ├── routes/
│   │   ├── +layout.svelte         -- shell con sidebar de navegacion
│   │   ├── +page.svelte           -- dashboard de proyectos
│   │   ├── sprite-factory/        -- Auto-Sprite, Slicer, Pixel Art
│   │   ├── image-engine/          -- Image Gen, BG Removal, Outpainting
│   │   ├── audio-engine/          -- SFX, Music, Voice
│   │   ├── tile-forge/            -- Tile Gen, Seamless
│   │   ├── material-forge/        -- PBR Materials
│   │   ├── agent-engine/          -- Coding Agent
│   │   └── settings/              -- Model Config, API Keys
│   └── lib/
│       ├── components/            -- componentes reutilizables
│       ├── stores/                -- estado global
│       └── tauri/                 -- wrappers de APIs Tauri
│
├── src-tauri/                     -- Tauri app (Control Plane)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── migrations/                -- SQLite migrations
│   └── src/
│       ├── main.rs
│       ├── lib.rs                 -- entry point
│       ├── state.rs               -- AppState global
│       ├── setup.rs               -- inicializacion
│       └── commands/              -- Tauri IPC commands
│           ├── projects.rs
│           ├── assets.rs
│           ├── jobs.rs
│           └── settings.rs
│
├── crates/                        -- Control Plane crates
│   ├── sorceress-shared-kernel/   -- tipos primitivos compartidos
│   ├── sorceress-asset-management/
│   ├── sorceress-job-queue/
│   ├── sorceress-model-config/
│   ├── sorceress-identity/
│   └── sorceress-publishing/
│
└── workers/                       -- Data Plane (procesos independientes)
    ├── sprite-worker/
    ├── image-worker/
    ├── audio-worker/
    ├── video-worker/
    ├── tile-worker/
    ├── material-worker/
    └── agent-worker/
```

**Dependencias principales del workspace**:

```toml
[workspace.dependencies]
tokio       = { version = "1",   features = ["full"] }
serde       = { version = "1",   features = ["derive"] }
serde_json  = "1"
anyhow      = "1"
thiserror   = "2"
tracing     = "0.1"
uuid        = { version = "1",   features = ["v4", "serde"] }
chrono      = { version = "0.4", features = ["serde"] }
sqlx        = { version = "0.8", features = ["sqlite", "runtime-tokio", "migrate", "json", "uuid", "chrono"] }
```

**Dependencias del crate Tauri (`src-tauri`)**:

```toml
[dependencies]
tauri                     = { version = "2", features = ["tray-icon", "protocol-asset"] }
tauri-plugin-fs           = "2"
tauri-plugin-dialog       = "2"
tauri-plugin-shell        = "2"   # sidecars
tauri-plugin-sql          = { version = "2", features = ["sqlite"] }
tauri-plugin-store        = "2"
tauri-plugin-updater      = "2"
tauri-plugin-http         = "2"
tauri-plugin-notification = "2"
tauri-plugin-window-state = "2"
tauri-plugin-os           = "2"
axum        = "0.8"               # HTTP server para workers
notify      = "7"                 # file watching
sha2        = "0.10"
keyring     = "2"                 # OS keychain para API keys
```

**Integracion con Tauri v2**: El archivo `src-tauri/src/lib.rs` es el entry point de la aplicacion (anotado con `#[cfg_attr(mobile, tauri::mobile_entry_point)]`). Los Tauri commands son funciones async en Rust que reciben el `tauri::AppHandle` y el estado global `tauri::State<AppState>`, y devuelven `Result<T, String>`. Los eventos se emiten con `app.emit("event-name", payload)`. Los Channels se usan para streaming de datos pesados (tokens de LLM, chunks de imagen durante generacion).

**Cambio critico de Tauri v1 a v2**: Los permisos ya no se declaran en `allowlist` sino en `capabilities/*.json`. Cada ventana tiene permisos expliciticos. Los commands en el modulo `lib.rs` no pueden ser `pub` (limitacion del codegen de Tauri); en modulos separados si pueden.

### 5.3 Base de Datos

**Motor**: SQLite via `sqlx` con migraciones en `src-tauri/migrations/`.

**Schema inicial**:

```sql
-- 0001_initial.sql

CREATE TABLE projects (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    root_path   TEXT NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP,
    settings    JSON
);

CREATE TABLE assets (
    id              TEXT PRIMARY KEY,
    project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    collection_id   TEXT REFERENCES collections(id),
    kind            TEXT NOT NULL, -- Image|Audio|Video|SpriteSheet|Tileset|Material|Code
    name            TEXT NOT NULL,
    tags            JSON DEFAULT '[]',
    import_source   TEXT NOT NULL, -- uploaded|generated|derived
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE asset_versions (
    id              TEXT PRIMARY KEY,
    asset_id        TEXT NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    storage_path    TEXT NOT NULL,
    checksum        TEXT NOT NULL,
    size_bytes      INTEGER,
    width_px        INTEGER,
    height_px       INTEGER,
    duration_ms     INTEGER,
    format          TEXT,
    derived_from    TEXT REFERENCES asset_versions(id),
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE collections (
    id          TEXT PRIMARY KEY,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    created_at  DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE jobs (
    id              TEXT PRIMARY KEY,
    worker_kind     TEXT NOT NULL,
    operation       TEXT NOT NULL,
    params          JSON NOT NULL,
    priority        INTEGER NOT NULL DEFAULT 50,
    status          TEXT NOT NULL DEFAULT 'Pending',
    worker_id       TEXT,
    result          JSON,
    error           TEXT,
    retries         INTEGER DEFAULT 0,
    max_retries     INTEGER DEFAULT 2,
    dependencies    JSON DEFAULT '[]',
    submitted_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
    started_at      DATETIME,
    completed_at    DATETIME
);

CREATE TABLE model_profiles (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    provider        TEXT NOT NULL, -- Local|Cloud
    model_id        TEXT NOT NULL,
    capabilities    JSON NOT NULL,
    params          JSON NOT NULL,
    quality_tier    TEXT NOT NULL, -- Draft|Standard|High|Ultra
    cost_per_unit   JSON
);

CREATE TABLE routing_rules (
    id                  TEXT PRIMARY KEY,
    operation_pattern   TEXT NOT NULL,
    target_profile_id   TEXT NOT NULL REFERENCES model_profiles(id),
    fallback_chain      JSON DEFAULT '[]',
    conditions          JSON DEFAULT '[]',
    priority            INTEGER DEFAULT 0
);

-- Indices
CREATE INDEX idx_assets_project ON assets(project_id);
CREATE INDEX idx_asset_versions_asset ON asset_versions(asset_id);
CREATE INDEX idx_jobs_status ON jobs(status, worker_kind);
CREATE INDEX idx_jobs_submitted ON jobs(submitted_at DESC);
```

### 5.4 Procesamiento de IA

**Estrategia por tamano de modelo**:

| Tamano modelo | Estrategia | Crate Rust |
|---------------|------------|------------|
| < 500 MB | En proceso Tauri (candle o ort) | `candle-core` o `tract-onnx` |
| 500 MB - 2 GB | En proceso con candle | `candle-core`, `candle-transformers` |
| 2 GB - 8 GB | Sidecar worker + REST API local | `axum` (worker), `reqwest` (cliente) |
| > 8 GB | Sidecar + opcion ComfyUI externo | `reqwest` hacia localhost:8188 |
| LLM GGUF | llama.cpp sidecar | `llama-cpp-rs` o subprocess |

**Inferencia local con candle**:

```toml
candle-core         = "0.9"
candle-nn           = "0.9"
candle-transformers = "0.9"
hf-hub              = { version = "0.3", features = ["tokio"] }
tokenizers          = "0.21"
```

Soporte de backends: CPU (MKL/Accelerate), CUDA (NVIDIA), Metal (Apple Silicon). Los modelos se descargan automaticamente desde HuggingFace Hub en el primer uso y se cachean en el directorio de datos de la aplicacion (~/.local/share/sorceress/models/ en Linux).

**Inferencia con ONNX Runtime (ort)**:

```toml
ort = { version = "2", features = ["cuda", "tensorrt"] }
```

Usado principalmente para BiRefNet (background removal) y Kokoro-82M (TTS), ambos disponibles como exports ONNX en HuggingFace. `ort` es mas maduro para modelos de vision/audio que `candle`.

**Fallback strategy**:

```
1. Comprobar si hay modelo local descargado y VRAM suficiente
2. Si no → usar API cloud configurada para esa operacion (Model Config routing rules)
3. Si no hay API key configurada → mostrar dialogo al usuario con opciones:
   a. Configurar API key (link a settings)
   b. Descargar modelo local (con estimacion de tamano y tiempo)
   c. Cancelar operacion
```

---

## 6. Flujo de Datos por Feature

### 6.1 Auto-Sprite (Video → Sprite Sheet)

**Descripcion**: El usuario arrastra un video MP4 de un walk cycle de personaje al panel Auto-Sprite. Quiere un spritesheet a 12 FPS para Godot.

```
┌────────────────────────────────────────────────────────────────┐
│ FRONTEND                                                        │
│  1. Drag & drop de video.mp4 → componente DropZone             │
│  2. Preview del primer frame en canvas PixiJS                   │
│  3. UI: configurar FPS=12, dedup=0.92, meta=Godot, format=PNG  │
│  4. Click "Generate" → invoke('auto_sprite_command', params)    │
└─────────────────────────────┬──────────────────────────────────┘
                              │ Tauri IPC command
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ CONTROL PLANE (src-tauri)                                       │
│                                                                 │
│  auto_sprite_command(video_path, config):                       │
│    1. asset-management: ImportAsset(video.mp4) → asset_id="a1" │
│    2. model-config: ResolveModel("autosprite.*")                │
│       → no model needed (CPU/GPU processing local)             │
│    3. job-queue: SubmitJob(                                     │
│         worker_kind: SpriteFactory,                             │
│         operation: "autosprite.generate",                       │
│         params: { source: "a1", fps: 12, dedup: 0.92,         │
│                   format: "PNG", meta: "Godot" },              │
│         priority: High                                          │
│       ) → job_id="j1"                                          │
│    4. Returns job_id="j1" al frontend                           │
│    5. Frontend muestra progress bar ligada a job_id            │
└─────────────────────────────┬──────────────────────────────────┘
                              │ IPC socket dispatch
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ DATA PLANE — sprite-worker                                      │
│                                                                 │
│  PASO 1 — Frame Extraction (0% → 30%)                          │
│    ffmpeg-next: extraer frames a 12 FPS → Vec<RgbaImage>       │
│    Deduplication: phash perceptual → eliminar frames con       │
│    similitud > 0.92 → resultado: 48 frames unicos              │
│    IPC progress: "Extracting frames: 32/60"                    │
│                                                                 │
│  PASO 2 — Atlas Packing (30% → 70%)                            │
│    crunch::pack_rects(frame_dimensions) → layout optimo        │
│    Render: pegar cada frame en la posicion del atlas           │
│    Resultado: spritesheet.png de 512x512 pixels                │
│                                                                 │
│  PASO 3 — MetaFile Generation (70% → 95%)                      │
│    Godot .tres format: generar SpriteFrames resource           │
│    AnimationClip "default": frames 0..47 a 12 FPS             │
│    Bounding boxes y pivot points por frame                     │
│                                                                 │
│  PASO 4 — Write outputs (95% → 100%)                           │
│    Escribe: assets/generated/spritesheet_a1.png                │
│    Escribe: assets/generated/spritesheet_a1.tres               │
│                                                                 │
│  IPC: job_completed { output_paths: [png, tres],               │
│                       metadata: { frame_count: 48 } }          │
└─────────────────────────────┬──────────────────────────────────┘
                              │ job_completed via IPC
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ CONTROL PLANE — reaccion al completion                          │
│                                                                 │
│  job-queue: update_status(j1, Completed)                        │
│  asset-management: CreateDerivedAsset(source="a1",              │
│    path="spritesheet.png", kind=SpriteSheet) → asset_id="a2"   │
│  asset-management: CreateDerivedAsset(source="a1",              │
│    path="spritesheet.tres", kind=Code) → asset_id="a3"        │
│  tauri::emit("job_completed", { job_id:"j1",                   │
│    assets: ["a2", "a3"] })                                     │
└─────────────────────────────┬──────────────────────────────────┘
                              │ Tauri event
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ FRONTEND — post-procesamiento                                   │
│                                                                 │
│  Renderiza spritesheet en PixiJS con grid overlay              │
│  Muestra AnimationClips detectados con preview animado         │
│  Botones: "Add to Collection", "Export", "Re-slice"            │
│  El .tres se puede previsualizar como copia de Godot embed     │
└────────────────────────────────────────────────────────────────┘
```

**Modelos/algoritmos usados**: FFmpeg (extraccion de frames), pHash (deduplicacion perceptual), algoritmo de rectangle packing (crunch), ninguna IA (procesamiento clasico). Tiempo estimado: 5-30 segundos segun duracion del video.

**Contexts participantes**: Asset Management, Job Queue, Sprite Factory.

### 6.2 Image Generation (Prompt → Game Art)

**Descripcion**: El usuario escribe "medieval knight, transparent background" y quiere 4 variantes en estilo pixel art de 256x256.

```
┌────────────────────────────────────────────────────────────────┐
│ FRONTEND                                                        │
│  1. Escribe prompt en campo de texto                           │
│  2. Selecciona preset "Pixel Art Style", batch=4, 256x256      │
│  3. Elige modelo: "Local - FLUX schnell Q4" o "Cloud - fal.ai" │
│  4. Click "Generate" → invoke('image_gen_command', params)     │
└─────────────────────────────┬──────────────────────────────────┘
                              │ Tauri IPC command
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ CONTROL PLANE                                                   │
│                                                                 │
│  PASO 1 — Resolver modelo                                       │
│  model-config: ResolveModel("imagegen.txt2img",                 │
│    hints: { quality: Standard, style: PixelArt })              │
│  → ModelProfile { provider: Local,                             │
│      model_id: "flux-schnell-q4",                              │
│      params: { steps: 4, guidance: 0.0 } }                    │
│                                                                 │
│  PASO 2 — Enriquecer prompt (ACL: UI → domain)                 │
│  preset "Pixel Art" agrega automaticamente:                    │
│    positive: "..., pixel art, 8-bit, sprite, game asset"       │
│    negative: "blurry, photorealistic, 3D render"               │
│                                                                 │
│  PASO 3 — Submit job                                            │
│  job-queue: SubmitJob(ImageEngine, "imagegen.txt2img",         │
│    { prompt, negative_prompt, model_profile_id,                │
│      batch_size: 4, width: 256, height: 256 },                 │
│    priority: High) → job_id="j2"                               │
└─────────────────────────────┬──────────────────────────────────┘
                              │ IPC dispatch
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ DATA PLANE — image-worker                                       │
│                                                                 │
│  PASO 1 — Seleccionar backend                                   │
│  model_profile.provider = Local →                              │
│    Si VRAM >= 8 GB: usa candle con FLUX schnell Q4             │
│    Sino: switch a ComfyUI local API (localhost:8188)           │
│    Sino: fallback a fal.ai API ($0.003/imagen)                 │
│                                                                 │
│  PASO 2 — Inferencia (0% → 90%)                                │
│  FLUX schnell: 4 pasos de denoising para cada imagen           │
│  Channels Tauri: streaming de progreso de generacion           │
│  Genera 4 imagenes PNG de 256x256 con seeds distintos          │
│                                                                 │
│  PASO 3 — Guardar outputs                                       │
│  Escribe: assets/generated/gen_{seed}.png (x4)                 │
│                                                                 │
│  IPC: job_completed { generations: [                           │
│    { seed: 42, path: "gen_42.png" },                           │
│    { seed: 1337, path: "gen_1337.png" }, ... ] }               │
└─────────────────────────────┬──────────────────────────────────┘
                              │ Completado
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ FRONTEND                                                        │
│                                                                 │
│  Muestra 4 imagenes en grid con selector                       │
│  Botones por imagen: "Select", "Remove BG", "Variations"       │
│  "Remove BG" → nuevo job para image-worker ("imagegen.rembg")  │
│  "Add to Collection" → asset-management: ImportAsset           │
└────────────────────────────────────────────────────────────────┘
```

**Modelos usados**: FLUX.1-schnell (local, Apache 2.0) o fal.ai API como fallback. Para Remove BG: BiRefNet via ONNX Runtime. Tiempo estimado: 3-15 segundos para 4 imagenes en GPU dedicada.

**Contexts participantes**: Model Config, Job Queue, Image Engine, Asset Management.

### 6.3 AI Coding Agent

**Descripcion**: El usuario quiere crear un sistema de particulas para un hechizo de fuego en Godot 4. Escribe la tarea en lenguaje natural.

```
┌────────────────────────────────────────────────────────────────┐
│ FRONTEND                                                        │
│  1. Abre panel "Coding Agent"                                  │
│  2. Escribe: "Create a particle system for a fireball spell    │
│     effect in Godot 4 GDScript, burst emission, fade 2s"      │
│  3. (Opcional) Adjunta archivos del proyecto como contexto     │
│  4. Click "Start" → invoke('start_agent_session', params)     │
└─────────────────────────────┬──────────────────────────────────┘
                              │ Tauri IPC command
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ CONTROL PLANE                                                   │
│                                                                 │
│  model-config: ResolveModel("agent.plan",                       │
│    hints: { complexity: High, task_type: GDScript })           │
│  → ModelProfile { provider: Cloud,                             │
│      model_id: "claude-3-7-sonnet" }                          │
│                                                                 │
│  job-queue: SubmitJob(AgentEngine, "agent.start_session",      │
│    { task, context_files, model_profile_id },                  │
│    priority: Critical) → job_id="j3", session_id="s1"         │
└─────────────────────────────┬──────────────────────────────────┘
                              │ IPC dispatch
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ DATA PLANE — agent-worker                                       │
│                                                                 │
│  === FASE 1: PLANNING ===                                      │
│                                                                 │
│  PlanGenerator → LlmPort (ACL): construye system prompt        │
│  + historial de conversacion + archivos de contexto            │
│  → POST Anthropic API /messages (streaming)                    │
│  → LLM genera Plan:                                            │
│    Step 1: Create fireball_particles.gd                        │
│    Step 2: Create fireball.tscn referencing the script         │
│    Step 3: Generate particle texture asset (64x64)             │
│    Step 4: Validate scene loads without errors                 │
│                                                                 │
│  IPC emit: PlanGenerated { steps: [...] }                      │
│  → Frontend renderiza el plan como checklist visible           │
│                                                                 │
│  === FASE 2: EXECUTION ===                                     │
│                                                                 │
│  Step 1: StepExecutor:                                         │
│  → LLM genera GDScript para GPUParticles2D                     │
│  → WorkspacePort::write_file("scripts/fireball_particles.gd")  │
│  → CodeDiffer: genera diff coloreado                           │
│  → CheckpointManager: snapshot del workspace                   │
│  IPC emit: StepCompleted { step_id: 1, diff, thought }        │
│                                                                 │
│  [Pausa si modo "manual approval" — espera UserApproved]       │
│                                                                 │
│  Step 3: necesita textura de particula →                       │
│  JobQueuePort::submit_job(                                     │
│    ImageEngine, "imagegen.txt2img",                            │
│    { prompt: "fireball particle, circular glow, 64x64,        │
│               orange yellow, transparent background" }         │
│  ) → delega al Image Engine Context                            │
│  Espera result y usa el asset en el .tscn generado            │
│                                                                 │
│  === FASE 3: PREVIEW ===                                       │
│                                                                 │
│  PreviewPort::render(workspace_root):                          │
│  → Godot headless --export-debug para validar escena           │
│  → Si hay errores de parseo: LLM para diagnostico y fix        │
│  → Si OK: screenshot del preview                               │
│                                                                 │
│  IPC emit: SessionCompleted { files, preview_screenshot }      │
└─────────────────────────────┬──────────────────────────────────┘
                              │ SessionCompleted
                              ▼
┌────────────────────────────────────────────────────────────────┐
│ CONTROL PLANE + FRONTEND                                        │
│                                                                 │
│  asset-management: ImportAsset(.gd, .tscn como kind=Code)     │
│  tauri::emit("session_completed", { files, preview })          │
│  Frontend: muestra codigo con syntax highlighting, diff view,  │
│  boton "Copy to Project", historial de checkpoints navegables  │
└────────────────────────────────────────────────────────────────┘
```

**Modelos usados**: Claude Sonnet 4.6 (API, tier Pro) o Qwen3 14B GGUF (local, tier Free). Image Engine para assets visuales delegados. tree-sitter para validacion de AST del codigo generado.

**Contexts participantes**: Model Config, Job Queue, Agent Engine, Image Engine (delegado), Asset Management. Eventos generados: 12+ a lo largo del flujo completo.

---

## 7. UX y Diseno de Interfaz

### 7.1 Layout principal

```
╔══════════════════════════════════════════════════════════════════════╗
║  TITLE BAR (nativa OS + controls de ventana)                        ║
╠══════════════╦══════════════════════════════════════╦═══════════════╣
║              ║                                      ║               ║
║  SIDEBAR     ║         AREA DE TRABAJO CENTRAL      ║  PANEL DE     ║
║  (240px)     ║                                      ║  PROPIEDADES  ║
║  ──────────  ║  ┌──────────────────────────────┐   ║  (300px)      ║
║  Proyectos   ║  │                              │   ║  ──────────── ║
║  ─────────── ║  │   Canvas Principal           │   ║  Parametros   ║
║  > Proyecto1 ║  │   (PixiJS / Konva)           │   ║  del tool     ║
║  > Proyecto2 ║  │                              │   ║  activo       ║
║              ║  │   Preview / Editor           │   ║               ║
║  Herramientas║  │   interactivo                │   ║  Modelo IA:   ║
║  ─────────── ║  │                              │   ║  [selector]   ║
║  [S] Sprites ║  └──────────────────────────────┘   ║               ║
║  [I] Images  ║                                      ║  Quality:     ║
║  [A] Audio   ║  ┌──────────────────────────────┐   ║  [Draft|Std|  ║
║  [V] Video   ║  │  BARRA DE HERRAMIENTAS /      │   ║   High|Ultra] ║
║  [T] Tiles   ║  │  TABS de operaciones          │   ║               ║
║  [M] Materia ║  └──────────────────────────────┘   ║  Historia:    ║
║  [C] Coding  ║                                      ║  [job list]   ║
║              ║  AREA DE PROMPT / CONFIGURACION      ║               ║
║  Ajustes     ║  (abajo, colapsable)                 ║               ║
║  ─────────── ║                                      ║               ║
║  API Keys    ║                                      ║               ║
║  Modelos     ║                                      ║               ║
╠══════════════╩══════════════════════════════════════╩═══════════════╣
║  STATUS BAR: worker status | queue depth | VRAM usage | job status  ║
╚══════════════════════════════════════════════════════════════════════╝
```

**Sidebar** (240px, colapsable): Navegacion jerarquica por proyectos y herramientas. Iconos + texto. Los proyectos recientes estan en la parte superior. Las herramientas estan agrupadas por categoria. Ajustes en la parte inferior.

**Area de trabajo central**: El canvas principal ocupa el 80% del espacio disponible. Dependiendo del tool activo muestra: un canvas PixiJS para sprites/animaciones, un grid de imagenes para generacion en batch, un editor de codigo con syntax highlighting para el coding agent, o una interfaz especifica del tool.

**Panel de propiedades** (300px, colapsable): Contextual al tool activo. Muestra los parametros de configuracion del tool, el selector de modelo/calidad, el historial de jobs recientes con su estado, y accesos rapidos a la asset library.

**Barra de estado**: Indicadores siempre visibles: estado de los workers (activo/inactivo), profundidad de la job queue (X jobs pendientes), uso de VRAM, y progreso del job activo actual.

### 7.2 Patrones de interaccion

**Drag and drop de archivos**: Todas las herramientas que aceptan input de archivo soportan drag and drop desde el explorador de archivos del OS. Implementado con el evento `ondragover`/`ondrop` en el componente frontend + validacion del tipo de archivo en Rust antes de crear el job.

**Preview en tiempo real**: Para operaciones de bajo costo computacional (pixel art, background removal con modelos ligeros), el preview se actualiza en tiempo real mientras el usuario ajusta parametros. Para operaciones pesadas (generacion de imagen), se muestra un preview aproximado o placeholder mientras el job corre en el worker.

**Progreso de jobs**: Cada job tiene un indicador de progreso visible en el panel de propiedades y en la status bar. El progreso es streaming desde el worker via el protocolo IPC. Cuando hay multiples jobs en paralelo, se muestra un mini-dashboard con cada job y su porcentaje.

**Cancelacion**: Todos los jobs pueden cancelarse via boton "Cancel" que envia un token de cancelacion al worker. Los workers comprueban la cancelacion entre pasos del pipeline. Los checkpoints del Agent Engine permiten rollback a cualquier estado previo, no solo cancelacion.

**Undo/redo**: Para el editor de sprites y tiles, undo/redo classico (historial de acciones en memoria, limitado a 50 estados). Para operaciones de generacion IA, el "undo" es volver a la version anterior del asset via asset versions. No se re-ejecuta la IA al hacer undo.

### 7.3 Responsive y multi-ventana

**Single window con panels dockables**: La aplicacion opera por defecto en una sola ventana con layout de panels. Tauri v2 soporta multiwebview experimental (feature flag `unstable`) que permite paneles completamente independientes dentro de la misma ventana. Para el MVP, layout fijo con panels colapsables es suficiente.

**Multi-ventana**: Tauri v2 soporta multiples `WebviewWindow` nativas. El caso de uso principal es: ventana principal para la herramienta activa + ventana secundaria para la asset library. Implementar como feature de fase 2.

**Responsive**: El layout usa Tailwind CSS con breakpoints para soportar ventanas de diferente tamano. El panel de propiedades y el sidebar se colapsan automaticamente a menos de 1200px de ancho. A menos de 800px de ancho, los panels se apilan verticalmente.

**Persistencia de layout**: `tauri-plugin-window-state` guarda y restaura automaticamente el tamano, posicion y estado de maximizado de la ventana entre sesiones. El estado de los panels (abiertos/cerrados) se persiste en `tauri-plugin-store` (clave-valor simple).

---

## 8. Roadmap de Desarrollo

### Fase 0: Fundaciones (semanas 1-2)

**Objetivo**: Tener el esqueleto funcional de la aplicacion corriendo.

| Feature | Descripcion | Entregable |
|---------|-------------|------------|
| Setup Tauri v2 | Proyecto con SvelteKit o React + Vite + Tailwind | App que arranca |
| Layout principal | Sidebar, area de trabajo, panel de propiedades | Shell navegable |
| SQLite + migraciones | Tablas de projects, assets, jobs | DB funcional |
| Project Manager | CRUD de proyectos, selector en sidebar | Proyectos creables |
| Asset browser basico | Lista de assets con previews en miniatura | Assets navegables |
| Model Config UI | Panel de configuracion de API keys y modelos | Keys configurables |

**Dependencias tecnicas**: Rust toolchain, Node.js, Tauri CLI v2, sqlx-cli para migraciones.

**Deliverable**: Aplicacion que corre en los 3 OS, permite crear proyectos y navegar el filesystem. Sin IA todavia.

### Fase 1: Core Tools (semanas 3-6)

**Objetivo**: Implementar las herramientas de procesamiento de assets sin IA (o con IA simple via API).

| Feature | Semana | Descripcion | Dependencia |
|---------|--------|-------------|-------------|
| Sprite Sheet Slicer | 3 | Grid slicer + auto-detect por bounding box | Fase 0 |
| Pixel Art Converter | 3-4 | Pipeline clasico: nearest-neighbor + Wu + Floyd-Steinberg | Fase 0 |
| Auto-Sprite (sin IA) | 4-5 | FFmpeg frame extraction + dedup + atlas packing | Fase 0 |
| Background Removal (API) | 5 | BiRefNet via fal.ai API (sin modelo local todavia) | Fase 0 |
| Image Gen (API) | 5-6 | FLUX schnell via fal.ai API | Fase 0 |
| Job Queue UI | 4 | Progress bar, cancel, historial de jobs | Fase 0 |

**Deliverable**: Herramientas de sprites completamente funcionales. Image Gen via API de pago funcional. Job Queue con UI de progreso.

**Hito de calidad**: El pipeline Auto-Sprite debe producir un spritesheet valido para Godot a partir de cualquier video MP4 de menos de 30 segundos.

### Fase 2: IA Integration (semanas 7-10)

**Objetivo**: Integrar modelos de IA locales para las herramientas principales.

| Feature | Semana | Modelo | VRAM req |
|---------|--------|--------|----------|
| Background Removal local | 7 | BiRefNet ONNX via ort | 2-4 GB |
| TTS / Voice local | 7-8 | Kokoro-82M ONNX | CPU |
| SFX Generation local | 8-9 | AudioLDM 2 via candle | 8 GB |
| Image Gen local (FLUX schnell) | 9-10 | FLUX schnell Q4 via candle | 8 GB |
| Descarga de modelos on-demand | 7 | hf-hub + progress via Channel | — |
| Sidecar infrastructure | 7 | ai-worker con axum REST API | — |

**Deliverable**: Modelos locales operativos para background removal, TTS y SFX. Image Gen local en GPU con 8+ GB VRAM. Sistema de descarga de modelos con progreso.

**Nota critica**: La integracion de FLUX con candle requiere cuidado con los pesos de CLIP + T5-XXL + flux-transformer. Verificar que la cadena de dependencias candle-transformers soporta FLUX a la version del MVP antes de comenzar.

### Fase 3: Coding Agent (semanas 11-14)

**Objetivo**: Implementar el AI Coding Agent completo.

| Tarea | Semana | Descripcion |
|-------|--------|-------------|
| agent-worker skeleton | 11 | Proceso sidecar con IPC, loop de mensajes |
| LLM streaming (API) | 11 | Claude / OpenAI API con streaming de tokens |
| Workspace isolation | 12 | Sandbox filesystem, lectura de archivos de contexto |
| Plan generation + UI | 12 | Generar plan, renderizarlo como checklist en frontend |
| Step execution + diffs | 13 | Ejecutar ToolCalls, generar diffs, mostrar en UI |
| Checkpoints + rollback | 13 | Snapshots del workspace, navegacion por historial |
| Delegacion a Image Engine | 14 | Agent puede pedir assets al Image Engine via Job Queue |
| LLM local (Qwen3 14B) | 14 | llama.cpp sidecar con GGUF Q4 para tier gratuito |

**Deliverable**: Coding agent funcional que puede generar scripts GDScript y C# simples, con diffs coloreados, checkpoints navegables, y delegacion de assets al Image Engine.

**Riesgo tecnico**: El preview de Godot (headless) puede ser fragil. Mitigacion: hacer el preview opcional y no bloquear el flujo si falla la validacion headless.

### Fase 4: Publishing y Polish (semanas 15-18)

**Objetivo**: Herramientas restantes, distribucion y calidad.

| Tarea | Semana | Descripcion |
|-------|--------|-------------|
| Tile Generator | 15 | noise + WFC + seamless pipeline |
| Seamless Texture Gen | 15-16 | Mirror padding + ControlNet tile via API |
| PBR Material Generator | 16 | Pipeline hibrido: albedo (FLUX) + mapas derivados |
| Image-to-Video | 16 | fal.ai Wan 2.5 / Kling API |
| Quick Sprites pipeline | 17 | Video Gen → Auto-Sprite en un paso |
| Music Generation | 17 | MIDI procedural + Suno API opcional |
| Auto-updater | 18 | tauri-plugin-updater + endpoint de releases |
| Code signing | 18 | macOS Developer ID + Windows EV cert |
| CI/CD GitHub Actions | 18 | Build multiplataforma + auto-release |
| Publishing Context (basico) | 18 | Empaquetado HTML5 + link a itch.io |

**Deliverable**: Aplicacion completa lista para distribucion en AppImage (Linux), DMG (macOS) y MSI (Windows). Todas las features del inventario implementadas excepto el arcade embed completo.

---

## 9. Riesgos y Mitigaciones

| Riesgo | Probabilidad | Impacto | Mitigacion |
|--------|-------------|---------|------------|
| WebKitGTK en Linux no soporta API CSS/JS moderna | Alta | Alto | Testear continuamente en Ubuntu LTS con webkit2gtk-4.1; evitar CSS Grid subgrid y otras features recientes sin polyfill |
| candle no soporta FLUX en la version disponible al momento de implementar | Media | Alto | Tener listo el fallback a ComfyUI como backend de difusion; monitorear releases de candle-transformers |
| VRAM insuficiente en maquinas de usuario (< 8 GB) | Alta | Medio | Todos los modelos tienen fallback a API cloud; las APIs cloud son transparentes para el usuario con costo visible |
| Worker sidecar se cae durante inferencia | Media | Alto | Heartbeat cada 10s; Job Queue relanza el worker automaticamente y reintenta el job (max 2 reintentos); el frontend muestra error claro si falla definitivamente |
| Modelo GGUF de LLM produce codigo incorrecto frecuentemente | Alta | Medio | UI de aprobacion por step (modo "manual approval" activado por defecto); rollback a checkpoint siempre disponible |
| API key del usuario se agota (quota o saldo) | Alta | Medio | Model Config muestra uso de quota en tiempo real; alertas cuando el usage supera el 80% del limite; fallback a modelo local si hay uno disponible |
| Tauri v2 multiwebview no es estable (feature flag `unstable`) | Alta | Bajo | No usar multiwebview en MVP; usar ventanas separadas nativas en su lugar |
| Dependencias nativas de CV (opencv-rust) complican el build | Media | Medio | Usar `image` + `imageproc` + `ort` puro Rust primero; opencv-rust solo si `ort` es insuficiente |
| Distribucion de binarios grandes (modelos IA) | Alta | Medio | Distribuir la app sin modelos; descargar on-demand con `hf-hub` en el primer uso; modelos cacheados en directorio de usuario |
| Licencia de Stable Audio Open supera $1M revenue | Baja | Alto | Cambiar a AudioLDM 2 (Apache 2.0) cuando el revenue se acerque al limite; documentar esto claramente |
| llama.cpp sidecar no se puede bundlear para todas las plataformas | Media | Medio | Compilar llama.cpp como sidecar por plataforma en CI; usar `externalBin` de Tauri con nombres de triple target |
| Privacidad de codigo del usuario en el Coding Agent | Baja | Alto | Tier gratuito usa solo modelos locales (Qwen3 14B GGUF); el envio a APIs cloud es opt-in explicito con aviso claro de privacidad |

---

## 10. Referencias

### Frameworks y runtime

- Tauri v2 documentacion oficial: https://v2.tauri.app
- Tauri migracion v1→v2: https://v2.tauri.app/start/migrate/from-tauri-1/
- candle (HuggingFace ML en Rust): https://github.com/huggingface/candle
- ort (ONNX Runtime bindings Rust): https://ort.pyke.io
- wgpu (GPU compute cross-platform): https://wgpu.rs
- axum (HTTP server Rust): https://github.com/tokio-rs/axum
- sqlx (async SQL para Rust): https://github.com/launchbadge/sqlx
- PixiJS (rendering 2D WebGL): https://pixijs.com
- shadcn/ui (componentes React): https://ui.shadcn.com
- Melt UI (componentes Svelte): https://melt-ui.com

### Modelos de IA open source

- FLUX.1-schnell (Apache 2.0, imagen): https://huggingface.co/black-forest-labs/FLUX.1-schnell
- FLUX.2 [pro] / [max] (Black Forest Labs, imagen): https://blackforestlabs.ai
- BiRefNet (MIT, background removal): https://huggingface.co/ZhengPeng7/BiRefNet
- SAM 2.1 (Apache 2.0, segmentacion): https://huggingface.co/facebook/sam2.1-hiera-large
- Kokoro-82M (Apache 2.0, TTS): https://huggingface.co/hexgrad/Kokoro-82M
- AudioLDM 2 (Apache 2.0, SFX): https://huggingface.co/cvssp/audioldm2
- Qwen3 72B / 14B (Apache 2.0, coding): https://huggingface.co/Qwen/Qwen3-14B
- Qwen2.5-Coder-32B (Apache 2.0, coding — referencia historica): https://huggingface.co/Qwen/Qwen2.5-Coder-32B-Instruct
- Llama 4 Scout / Maverick (Meta Llama 4 Community): https://huggingface.co/meta-llama
- CogVideoX-5B (Apache 2.0, video): https://huggingface.co/THUDM/CogVideoX-5b
- Wan2.1 (Apache 2.0, video): https://huggingface.co/Wan-AI/Wan2.1-T2V-14B
- Stable Audio Open (Stability AI, SFX): https://huggingface.co/stabilityai/stable-audio-open-1.0

### APIs cloud recomendadas

- fal.ai (FLUX, FLUX.2, video, background removal): https://fal.ai
- ElevenLabs (suite completa: TTS, SFX, Music, Voice Cloning — desde $6/mes): https://elevenlabs.io
- Anthropic Claude 4.x (Opus 4.6 / Sonnet 4.6 / Haiku 4.5 — coding agent): https://console.anthropic.com
- OpenAI GPT-5.4 / gpt-realtime-1.5: https://platform.openai.com
- DeepSeek-V3.2 API (coding alternativa economica con thinking integrado): https://platform.deepseek.com
- Suno (generacion de musica — Basic $8/mes): https://suno.com
- Meshy 5 (3D assets + PBR, plugin Godot): https://meshy.ai
- Tripo v3.0 Ultra (3D alta fidelidad): https://www.tripo3d.ai
- Stability AI API (imagen, materiales): https://platform.stability.ai

### Librerias Rust relevantes

- `image` (procesamiento de imagenes puro Rust): https://crates.io/crates/image
- `imageproc` (filtros, convoluciones, morfologia): https://crates.io/crates/imageproc
- `quantette` (cuantizacion de color Wu's algorithm): https://crates.io/crates/quantette
- `rodio` (audio playback): https://crates.io/crates/rodio
- `hound` (WAV I/O): https://crates.io/crates/hound
- `symphonia` (decodificacion multi-formato audio): https://crates.io/crates/symphonia
- `cpal` (cross-platform audio): https://crates.io/crates/cpal
- `fundsp` (sintesis audio procedural): https://crates.io/crates/fundsp
- `midly` (parse/write MIDI): https://crates.io/crates/midly
- `noise` (Perlin/Simplex/Worley noise): https://crates.io/crates/noise
- `crunch` (rectangle packing para atlas): https://crates.io/crates/crunch
- `ffmpeg-next` (bindings FFmpeg): https://crates.io/crates/ffmpeg-next
- `tree-sitter` (parsing incremental AST): https://crates.io/crates/tree-sitter
- `similar` (diff generation): https://crates.io/crates/similar
- `llama-cpp-rs` (bindings llama.cpp): https://crates.io/crates/llama-cpp-2
- `async-openai` (cliente OpenAI-compatible): https://crates.io/crates/async-openai
- `keyring` (OS keychain): https://crates.io/crates/keyring
- `notify` (file watching): https://crates.io/crates/notify
- `tauri-plugin-window-state`: https://crates.io/crates/tauri-plugin-window-state

### Referencias de diseno y arquitectura

- Domain-Driven Design (Eric Evans): conceptos de bounded contexts, ubiquitous language, context map
- Hexagonal Architecture (Alistair Cockburn): ports and adapters
- Wave Function Collapse: https://github.com/mxgmn/WaveFunctionCollapse
- PixelOE (pixel art extraction estado del arte 2024): https://github.com/KohakuBlueleaf/PixelOE
- Depth-Anything-V2 (estimacion de profundidad): https://huggingface.co/depth-anything/Depth-Anything-V2-Base

---

*Documento compilado el 14 de abril de 2026.*
*Basado en investigacion de modelos-ia-game-tools.md, arquitectura-tauri-v2.md y ddd-bounded-contexts.md.*
*Version 1.0 — autocontenido para iniciar construccion sin documentacion adicional.*
