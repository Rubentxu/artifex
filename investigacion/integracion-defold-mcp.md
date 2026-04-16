# Integración Defold — MCP Server para Sorceress

> Documento de investigación y diseño técnico para la integración entre la app Sorceress
> (Rust + Tauri v2) y el motor de juegos Defold via un MCP server en Rust.
> Fecha: 2026-04-14

---

## Índice

1. [Resumen Ejecutivo](#1-resumen-ejecutivo)
2. [Superficie de Integración de Defold](#2-superficie-de-integración-de-defold)
3. [Diseño del MCP Server `defold-mcp-server`](#3-diseño-del-mcp-server-defold-mcp-server)
4. [Comunicación Bidireccional](#4-comunicación-bidireccional)
5. [Integración con Bounded Context AssetPipeline](#5-integración-con-bounded-context-assetpipeline)
6. [Propuestas de Pensamiento Lateral](#6-propuestas-de-pensamiento-lateral)
7. [Plan de Implementación en 4 Fases](#7-plan-de-implementación-en-4-fases)

---

## 1. Resumen Ejecutivo

Sorceress es una suite de herramientas para game developers que combina IA generativa,
procesamiento de assets y orquestación de agentes en una aplicación de escritorio construida
con Rust y Tauri v2. El objetivo de esta integración es conectar Sorceress directamente con
el editor de Defold, permitiendo que los assets generados por la suite (sprites, tilemaps,
materiales, audio) lleguen al proyecto Defold sin fricción manual.

La integración se articula sobre tres mecanismos que Defold expone de forma nativa:

1. **Editor Scripts** en Lua que corren dentro del editor y pueden registrar endpoints HTTP.
2. **HTTP server interno** del editor (desde Defold 1.11.0), accesible en localhost.
3. **Bob CLI** (`bob.jar`) para builds y bundles desde línea de comandos.

El diseño propuesto introduce un **`defold-mcp-server`**, un servidor MCP implementado en
Rust que actúa como mediador entre Sorceress y el editor de Defold. El servidor expone tools
MCP que Sorceress invoca para ejecutar operaciones dentro del editor (importar assets, refrescar
atlas, lanzar builds, manipular colecciones de escena) y también recibe notificaciones del
editor vía HTTP callbacks cuando el usuario realiza acciones relevantes en Defold.

La integración se conecta al bounded context `AssetPipeline` de Sorceress, que es el
responsable de registrar, versionar y publicar los assets generados. El Job Queue actúa como
hub central: cuando un asset es generado por un worker de Sorceress, el Job Queue puede
despachar automáticamente un job de tipo `DefoldSyncJob` que el `defold-mcp-server` ejecuta.

---

## 2. Superficie de Integración de Defold

### 2.1 Editor Scripts

Los Editor Scripts son archivos Lua con extensión `.editor_script` que Defold carga al
iniciar el editor. Corren en el contexto del editor, tienen acceso a la API `editor.*` y
pueden reaccionar a comandos del menú, cambios en el proyecto, y eventos del editor.

**Capacidades disponibles:**

- Leer y modificar el grafo de recursos del proyecto (`editor.get`, `editor.set`).
- Ejecutar transacciones (`editor.transact`) para modificar recursos de forma atómica.
- Registrar comandos de menú personalizados (`editor.add_menu_command`).
- Registrar endpoints HTTP en el servidor interno del editor.
- Leer la selección actual en el editor.
- Acceder al árbol de recursos del proyecto.

**API del editor disponible en scripts:**

```lua
-- Leer una propiedad de un nodo en la escena
local value = editor.get(node_ref, "position")

-- Modificar una propiedad con transacción (undo-able)
editor.transact({
  editor.tx.set(node_ref, "position", vmath.vector3(100, 200, 0))
})

-- Añadir un comando al menú Editor
editor.add_menu_command({
  label = "Sorceress: Sync Assets",
  locations = { "Edit" },
  query = { selection = { type = "resource", cardinality = "one" } },
  active = function(opts) return true end,
  run = function(opts)
    -- lógica de sincronización
  end
})
```

**Eventos del editor a los que se puede suscribir:**

| Evento                     | Cuándo se dispara                                |
|----------------------------|--------------------------------------------------|
| `editor.on_build_started`  | Al iniciar una compilación                       |
| `editor.on_build_finished` | Al finalizar una compilación (éxito o fallo)     |
| `editor.on_resource_added` | Cuando se añade un recurso al proyecto           |
| `editor.on_selection_changed` | Cuando cambia la selección en el editor       |
| `editor.on_save`           | Cuando el usuario guarda el proyecto             |

---

### 2.2 HTTP Server del Editor y `get_http_server_routes()`

Desde Defold 1.11.0, el editor levanta un HTTP server en `localhost` en un puerto
configurable (por defecto suele ser `55555` pero es dinámico y se negocia al arrancar).

Los Editor Scripts pueden registrar endpoints adicionales implementando la función
`get_http_server_routes()`:

```lua
-- editor_script/sorceress_bridge.editor_script

local M = {}

-- Defold llama esta función para descubrir las rutas HTTP del script
function M.get_http_server_routes()
  return {
    {
      method = "POST",
      path   = "/sorceress/import-asset",
      handler = function(request)
        local body = json.decode(request.body)
        local asset_path = body.project_path
        local source_url  = body.source_url

        -- Descargar el archivo y copiarlo en el proyecto
        local ok = download_and_place(source_url, asset_path)

        if ok then
          -- Refrescar el recurso en el editor para que lo detecte
          editor.external_change(asset_path)
          return { status = 200, body = json.encode({ ok = true }) }
        else
          return {
            status = 500,
            body   = json.encode({ ok = false, error = "download failed" })
          }
        end
      end
    },
    {
      method = "GET",
      path   = "/sorceress/project-info",
      handler = function(request)
        local root = editor.get_project_root()
        return {
          status = 200,
          body   = json.encode({ root = root })
        }
      end
    },
    {
      method = "POST",
      path   = "/sorceress/rebuild-atlas",
      handler = function(request)
        local body = json.decode(request.body)
        local atlas_path = body.atlas_path

        -- Forzar re-compilación del atlas
        editor.build_resource(atlas_path)
        return { status = 200, body = json.encode({ ok = true }) }
      end
    }
  }
end

return M
```

El puerto del servidor HTTP se puede obtener programáticamente dentro del editor:

```lua
local port = editor.get_http_server_port()
-- También se escribe en un archivo de estado en el proyecto:
-- .internal/editor-server-port
```

Desde fuera del editor (por ejemplo, desde el `defold-mcp-server` en Rust), el puerto
se lee del archivo `.internal/editor-server-port` o se descubre vía un puerto fijo
configurado por el usuario.

---

### 2.3 Extensiones Nativas

Las extensiones nativas de Defold permiten añadir código C/C++ que se integra con el
runtime del motor. El flujo es:

1. Crear un directorio con el nombre de la extensión bajo `ext/<nombre>/`.
2. Poner el código C/C++ en `ext/<nombre>/src/`.
3. Declarar la extensión en `ext/<nombre>/ext.manifest`.
4. El build server de Defold (en la nube) compila la extensión para cada plataforma.

**Lifecycle de una extensión nativa:**

```c
// ext/sorceress_native/src/sorceress_native.cpp

#include <dmsdk/sdk.h>

// Función registrada como módulo Lua
static int SorceressNotify(lua_State* L) {
    const char* event_type = luaL_checkstring(L, 1);
    // Enviar notificación a la app Sorceress via HTTP
    // (usando la libnet de Defold o una librería embebida)
    DM_LOG_INFO("Sorceress event: %s", event_type);
    lua_pushboolean(L, 1);
    return 1;
}

static const luaL_reg sorceress_methods[] = {
    { "notify", SorceressNotify },
    { 0, 0 }
};

static dmExtension::Result AppInitializeSorceress(dmExtension::AppParams* params) {
    return dmExtension::RESULT_OK;
}

static dmExtension::Result InitializeSorceress(dmExtension::Params* params) {
    luaL_register(params->m_L, "sorceress", sorceress_methods);
    lua_pop(params->m_L, 1);
    return dmExtension::RESULT_OK;
}

DM_DECLARE_EXTENSION(SorceressNative, "SorceressNative",
    AppInitializeSorceress, nullptr,
    InitializeSorceress, nullptr, nullptr, nullptr)
```

La comunicación entre Lua y el código nativo sigue el protocolo estándar de la Lua C API.
Las extensiones nativas son útiles cuando se necesita rendimiento (por ejemplo, procesamiento
de assets en tiempo real dentro del runtime) pero son más complejas de mantener que los
Editor Scripts.

**Para la integración con Sorceress, el enfoque preferido es Editor Scripts + HTTP**, reservando
las extensiones nativas para el runtime del juego (no del editor).

---

### 2.4 Bob CLI

Bob es el compilador/bundler de Defold distribuido como `bob.jar`. Se ejecuta con Java y
permite automatizar el ciclo de build sin intervención del editor gráfico.

**Comandos principales:**

```bash
# Compilar el proyecto (sin bundlear)
java -jar bob.jar --root /ruta/al/proyecto build

# Compilar y generar bundle para una plataforma
java -jar bob.jar \
  --root /ruta/al/proyecto \
  --platform x86_64-macos \
  --bundle-output /ruta/output \
  bundle

# Compilar con extensiones nativas (requiere conexión al build server)
java -jar bob.jar \
  --root /ruta/al/proyecto \
  --email usuario@ejemplo.com \
  --auth <token> \
  build

# Limpiar artefactos de build
java -jar bob.jar --root /ruta/al/proyecto clean

# Verificar el proyecto (lint)
java -jar bob.jar --root /ruta/al/proyecto resolve
```

**Flags relevantes para automatización:**

| Flag                    | Descripción                                          |
|-------------------------|------------------------------------------------------|
| `--root`                | Directorio raíz del proyecto Defold                  |
| `--platform`            | Plataforma target (x86_64-linux, x86_64-macos, etc.) |
| `--bundle-output`       | Directorio donde depositar el bundle                 |
| `--variant`             | debug / release / headless                           |
| `--texture-compression` | Algoritmo de compresión de texturas                  |
| `--email` + `--auth`    | Credenciales para el build server cloud              |
| `--build-server`        | URL del build server (default: externo de Defold)    |
| `--archive`             | Generar archivo de recursos (liveupdate)             |

**Salida de Bob:**

Bob escribe los resultados en `stdout` / `stderr` con un formato legible. Las líneas de error
siguen el patrón `ERROR:RESOURCE:/ruta: mensaje`. El exit code es `0` en éxito y no cero en fallo.

---

### 2.5 Build Server y Extensiones Nativas en la Nube

Defold ofrece un build server gratuito en `build.defold.com` que compila extensiones nativas
para todas las plataformas soportadas. El flujo es:

```
Editor / Bob CLI
      │
      │ POST /build  (código fuente de la extensión)
      ▼
build.defold.com
      │
      │ Compilación cross-platform (Linux, macOS, Windows, Android, iOS, HTML5)
      ▼
.so / .dylib / .dll / .a
      │
      ▼
Editor (incorpora los binarios en el bundle)
```

Para instalar un build server propio (on-premise), Defold ofrece la imagen Docker
`defold/build_server`. Esto es relevante para estudios que no quieren enviar código al
exterior.

---

### 2.6 Asset Pipeline de Defold

Defold procesa los assets en el momento del build. El pipeline sigue este orden:

```
Archivo fuente (PNG, OGG, etc.)
      │
      ▼
Defold resource compiler
      │  ├─ .atlas  → sprite sheet packed (binario .texturec)
      │  ├─ .png    → .texturec (comprimido con ETC2/ASTC/etc.)
      │  ├─ .ogg    → .soundc (encapsulado)
      │  ├─ .lua    → .luac (bytecode)
      │  └─ .go     → .goc  (game object compilado)
      ▼
Artefactos en build/
      │
      ▼
Bundle (empaquetado por plataforma)
```

**Atlas (`.atlas`)**: Es el recurso más importante para sprites. Define qué imágenes se
empacan en un sprite sheet, animaciones por frames, y propiedades de padding. La compilación
de un atlas produce un `.texturec` y un `.texturesetc`.

```
/sprites/
  player.atlas      ← define el sprite sheet
  player_run_1.png  ← frames individuales
  player_run_2.png
  player_idle.png
```

**Implicación para Sorceress**: Cuando Sorceress genera un sprite sheet, debe escribir tanto
los PNGs individuales como el `.atlas` correspondiente en el proyecto Defold. El Editor Script
puede automatizar la creación del `.atlas` y luego forzar un refresco.

---

## 3. Diseño del MCP Server `defold-mcp-server`

### 3.1 Arquitectura General

El `defold-mcp-server` es un servidor MCP implementado en Rust que corre como proceso
independiente (sidecar) junto a Sorceress. Se comunica con el editor de Defold vía HTTP
al `localhost`, y expone sus capabilities al cliente MCP (Sorceress, agentes de IA, Claude
Desktop, etc.) via el protocolo MCP estándar sobre stdio o HTTP.

```
┌─────────────────────────────────────────────────────────┐
│                      Sorceress App                       │
│   (Tauri v2 + Rust backend)                              │
│                                                          │
│  ┌─────────────────┐         ┌──────────────────────┐   │
│  │  Job Queue      │────────►│  DefoldSync Worker   │   │
│  │  Context        │         │  (process sidecar)   │   │
│  └─────────────────┘         └──────────┬───────────┘   │
│                                         │ MCP calls      │
└─────────────────────────────────────────┼───────────────┘
                                          │
                          ┌───────────────▼───────────────┐
                          │      defold-mcp-server         │
                          │      (Rust, port 7777)         │
                          │                                │
                          │  MCP Protocol (JSON-RPC/HTTP)  │
                          └───────────────┬───────────────┘
                                          │ HTTP requests
                                          │ localhost:55555
                          ┌───────────────▼───────────────┐
                          │   Defold Editor HTTP Server    │
                          │   + Editor Scripts (Lua)       │
                          └───────────────────────────────┘
```

**Componentes del `defold-mcp-server`:**

- `McpServer`: loop principal que acepta conexiones MCP y despacha tools.
- `DefoldClient`: cliente HTTP que apunta al editor de Defold (`http://localhost:<port>`).
- `PortDiscovery`: lee `.internal/editor-server-port` o usa configuración estática.
- `ToolRegistry`: mapa de nombre de tool a handler.
- `BobRunner`: wrapper de `tokio::process::Command` para ejecutar `bob.jar`.
- `ProjectWatcher`: observa el filesystem del proyecto Defold y emite eventos.

**Stack Rust:**

```toml
[dependencies]
tokio          = { version = "1", features = ["full"] }
axum           = "0.7"           # HTTP server para el MCP endpoint
reqwest        = { version = "0.12", features = ["json"] }  # cliente HTTP para Defold
serde          = { version = "1", features = ["derive"] }
serde_json     = "1"
mcp-sdk        = "0.1"           # SDK MCP oficial (o implementación propia)
notify         = "6"             # filesystem watcher
tracing        = "0.1"
anyhow         = "1"
```

---

### 3.2 Protocolo MCP — JSON-RPC sobre HTTP

El servidor expone el protocolo MCP sobre HTTP en `http://localhost:7777/mcp`. Las llamadas
siguen el formato JSON-RPC 2.0:

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "method": "tools/call",
  "params": {
    "name": "defold_import_asset",
    "arguments": {
      "source_path": "/home/user/.sorceress/assets/goblin_run.png",
      "project_path": "/sprites/enemies/goblin_run.png",
      "atlas": "/sprites/enemies/goblin.atlas"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": "req-001",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"ok\":true,\"message\":\"Asset imported and atlas rebuilt\"}"
      }
    ]
  }
}
```

---

### 3.3 Catálogo de MCP Tools

#### Categoría: `asset_ops` — Operaciones sobre assets

**`defold_import_asset`**
Copia un asset generado por Sorceress al proyecto Defold y notifica al editor.

```
Nombre:      defold_import_asset
Descripción: Importa un asset (imagen, audio, fuente) al proyecto Defold activo.
             Copia el archivo a la ruta especificada dentro del proyecto y notifica
             al editor para que refresque el recurso.

Parámetros:
  source_path   : string  (requerido) — ruta absoluta al archivo fuente
  project_path  : string  (requerido) — ruta relativa dentro del proyecto Defold
                                         (ej: "/sprites/player/run.png")
  overwrite     : boolean (opcional, default: true) — sobreescribir si ya existe
  notify_editor : boolean (opcional, default: true) — llamar a editor.external_change

Respuesta:
{
  "ok": true,
  "project_path": "/sprites/player/run.png",
  "bytes_written": 45312,
  "editor_notified": true
}
```

**`defold_import_spritesheet`**
Importa un sprite sheet completo junto con su `.atlas` generado.

```
Nombre:      defold_import_spritesheet
Descripción: Importa un sprite sheet (PNG) y genera o actualiza el archivo .atlas
             correspondiente en el proyecto Defold.

Parámetros:
  source_png       : string   (requerido) — ruta al PNG del sprite sheet
  project_dir      : string   (requerido) — directorio destino en el proyecto
  atlas_name       : string   (requerido) — nombre del atlas (sin extensión)
  frame_width      : integer  (requerido) — ancho de cada frame en píxeles
  frame_height     : integer  (requerido) — alto de cada frame en píxeles
  animations       : array    (opcional)  — lista de { name, from_frame, to_frame, fps }
  margin           : integer  (opcional, default: 2)
  extrude_borders  : integer  (opcional, default: 2)

Respuesta:
{
  "ok": true,
  "atlas_path": "/sprites/player/player.atlas",
  "frames_imported": 8,
  "animations_created": ["run", "idle", "jump"]
}
```

**`defold_list_assets`**
Devuelve el árbol de recursos del proyecto Defold.

```
Nombre:      defold_list_assets
Descripción: Lista los recursos del proyecto Defold filtrando por tipo y directorio.

Parámetros:
  directory : string (opcional, default: "/") — directorio raíz del listado
  types     : array  (opcional) — filtros de extensión (ej: ["png", "atlas", "go"])
  recursive : boolean (opcional, default: true)

Respuesta:
{
  "ok": true,
  "assets": [
    { "path": "/sprites/player/run.png", "type": "png", "size": 45312 },
    { "path": "/sprites/player/player.atlas", "type": "atlas", "size": 1024 }
  ],
  "total": 2
}
```

---

#### Categoría: `build_ops` — Operaciones de compilación

**`defold_build`**
Lanza una compilación del proyecto vía Bob CLI.

```
Nombre:      defold_build
Descripción: Compila el proyecto Defold usando bob.jar. Reporta errores y warnings.

Parámetros:
  project_root : string  (requerido) — ruta al directorio raíz del proyecto
  variant      : string  (opcional, default: "debug") — "debug" | "release" | "headless"
  clean        : boolean (opcional, default: false) — limpiar antes de compilar

Respuesta:
{
  "ok": true,
  "exit_code": 0,
  "duration_ms": 3200,
  "errors": [],
  "warnings": ["WARNING:RESOURCE:/sprites/player.atlas: padding too small"]
}
```

**`defold_bundle`**
Genera un bundle para una plataforma específica.

```
Nombre:      defold_bundle
Descripción: Genera un bundle del proyecto para la plataforma indicada usando bob.jar.

Parámetros:
  project_root    : string (requerido)
  platform        : string (requerido) — "x86_64-linux" | "x86_64-macos" | "js-web" | ...
  output_dir      : string (requerido) — directorio donde depositar el bundle
  variant         : string (opcional, default: "release")
  bob_path        : string (opcional) — ruta a bob.jar si no está en PATH

Respuesta:
{
  "ok": true,
  "bundle_path": "/builds/my_game_x86_64-linux",
  "duration_ms": 15400,
  "platform": "x86_64-linux",
  "errors": []
}
```

**`defold_resolve_libraries`**
Descarga y resuelve las dependencias de librerías del proyecto.

```
Nombre:      defold_resolve_libraries
Descripción: Ejecuta `bob.jar resolve` para descargar las librerías declaradas en
             game.project.

Parámetros:
  project_root : string (requerido)

Respuesta:
{
  "ok": true,
  "libraries_resolved": 3,
  "duration_ms": 1800
}
```

---

#### Categoría: `editor_ops` — Operaciones sobre el editor

**`defold_get_project_info`**
Obtiene metadatos del proyecto Defold abierto en el editor.

```
Nombre:      defold_get_project_info
Descripción: Devuelve información sobre el proyecto Defold actualmente abierto en
             el editor: nombre, root, plataforma activa, versión de Defold.

Parámetros: (ninguno)

Respuesta:
{
  "ok": true,
  "project_name": "MyGame",
  "project_root": "/home/user/MyGame",
  "defold_version": "1.9.3",
  "editor_port": 55555
}
```

**`defold_hot_reload`**
Fuerza una recarga en caliente de uno o más recursos en el editor.

```
Nombre:      defold_hot_reload
Descripción: Notifica al editor de Defold que los recursos indicados han cambiado
             en disco y deben recargarse.

Parámetros:
  paths : array (requerido) — lista de rutas relativas al proyecto

Respuesta:
{
  "ok": true,
  "reloaded": ["/sprites/player/run.png", "/sprites/player/player.atlas"]
}
```

**`defold_get_selection`**
Devuelve la selección actual en el editor (nodo, recurso).

```
Nombre:      defold_get_selection
Descripción: Obtiene el recurso o nodo actualmente seleccionado en el editor de Defold.

Parámetros: (ninguno)

Respuesta:
{
  "ok": true,
  "selection_type": "resource",
  "path": "/sprites/player/player.atlas",
  "node_id": null
}
```

---

#### Categoría: `scene_ops` — Operaciones sobre colecciones y game objects

**`defold_add_game_object`**
Añade un game object a una colección existente.

```
Nombre:      defold_add_game_object
Descripción: Añade un game object (.go) a una colección (.collection) en el proyecto
             Defold con posición y propiedades iniciales.

Parámetros:
  collection_path : string (requerido) — ruta al .collection
  go_path         : string (requerido) — ruta al .go a añadir
  id              : string (opcional)  — identificador del nodo en la colección
  position        : object (opcional)  — { x: float, y: float, z: float }
  scale           : float  (opcional, default: 1.0)

Respuesta:
{
  "ok": true,
  "node_id": "goblin_enemy",
  "collection_path": "/main/level01.collection"
}
```

**`defold_update_node_property`**
Modifica una propiedad de un nodo en una colección.

```
Nombre:      defold_update_node_property
Descripción: Actualiza una propiedad de un nodo en una colección de Defold.
             Usa transacciones del editor para que la operación sea undo-able.

Parámetros:
  collection_path : string (requerido)
  node_id         : string (requerido)
  property        : string (requerido) — nombre de la propiedad (ej: "position")
  value           : any    (requerido) — nuevo valor

Respuesta:
{
  "ok": true,
  "node_id": "player_spawn",
  "property": "position",
  "old_value": { "x": 0, "y": 0, "z": 0 },
  "new_value": { "x": 128, "y": 64, "z": 0 }
}
```

---

#### Categoría: `extension_ops` — Operaciones sobre extensiones nativas

**`defold_install_extension`**
Añade una extensión nativa a las dependencias del proyecto.

```
Nombre:      defold_install_extension
Descripción: Añade una extensión nativa (URL de GitHub o archivo zip) a las
             dependencias declaradas en game.project y ejecuta resolve.

Parámetros:
  extension_url : string (requerido) — URL de la extensión (zip de GitHub release)
  project_root  : string (requerido)

Respuesta:
{
  "ok": true,
  "extension_url": "https://github.com/defold/extension-fbinstant/archive/refs/tags/4.5.0.zip",
  "dependency_added": true,
  "resolve_ok": true
}
```

**`defold_build_with_extensions`**
Compila el proyecto incluyendo extensiones nativas usando el build server.

```
Nombre:      defold_build_with_extensions
Descripción: Compila el proyecto usando bob.jar con credenciales para el build server
             cloud de Defold. Necesario cuando el proyecto tiene extensiones nativas.

Parámetros:
  project_root  : string (requerido)
  email         : string (requerido) — cuenta Defold para el build server
  auth_token    : string (requerido) — token de autenticación
  platform      : string (requerido)
  build_server  : string (opcional, default: "https://build.defold.com")

Respuesta:
{
  "ok": true,
  "duration_ms": 45000,
  "extensions_compiled": ["DefoldIAP", "DefoldFacebook"],
  "errors": []
}
```

---

### 3.4 Ejemplo de Implementación en Rust: Tool `defold_import_asset`

```rust
// src/tools/asset_ops.rs

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

use crate::defold_client::DefoldClient;

#[derive(Debug, Deserialize)]
pub struct ImportAssetParams {
    pub source_path: PathBuf,
    pub project_path: String,
    #[serde(default = "default_true")]
    pub overwrite: bool,
    #[serde(default = "default_true")]
    pub notify_editor: bool,
}

#[derive(Debug, Serialize)]
pub struct ImportAssetResult {
    pub ok: bool,
    pub project_path: String,
    pub bytes_written: u64,
    pub editor_notified: bool,
}

fn default_true() -> bool { true }

pub async fn defold_import_asset(
    params: ImportAssetParams,
    project_root: &PathBuf,
    defold_client: &DefoldClient,
) -> Result<ImportAssetResult> {
    // Validar que el archivo fuente existe
    let metadata = fs::metadata(&params.source_path)
        .await
        .with_context(|| format!("Source file not found: {:?}", params.source_path))?;

    // Construir la ruta de destino dentro del proyecto Defold
    // params.project_path es relativo, ej: "/sprites/player/run.png"
    let dest_path = project_root.join(
        params.project_path.trim_start_matches('/')
    );

    // Crear directorios intermedios si no existen
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create dir: {:?}", parent))?;
    }

    // Verificar overwrite
    if dest_path.exists() && !params.overwrite {
        anyhow::bail!("File already exists and overwrite=false: {:?}", dest_path);
    }

    // Copiar el archivo
    let bytes_written = fs::copy(&params.source_path, &dest_path)
        .await
        .with_context(|| format!("Failed to copy asset to {:?}", dest_path))?;

    // Notificar al editor de Defold via HTTP
    let editor_notified = if params.notify_editor {
        defold_client
            .notify_external_change(&params.project_path)
            .await
            .map(|_| true)
            .unwrap_or_else(|e| {
                tracing::warn!("Editor notification failed: {}", e);
                false
            })
    } else {
        false
    };

    Ok(ImportAssetResult {
        ok: true,
        project_path: params.project_path,
        bytes_written: metadata.len(),
        editor_notified,
    })
}
```

```rust
// src/defold_client.rs

use anyhow::Result;
use reqwest::Client;
use serde_json::json;

pub struct DefoldClient {
    client: Client,
    base_url: String,
}

impl DefoldClient {
    pub fn new(port: u16) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("http://localhost:{}", port),
        }
    }

    /// Lee el puerto del archivo .internal/editor-server-port del proyecto Defold
    pub async fn discover_port(project_root: &std::path::Path) -> Result<u16> {
        let port_file = project_root.join(".internal/editor-server-port");
        let content = tokio::fs::read_to_string(&port_file).await?;
        let port: u16 = content.trim().parse()?;
        Ok(port)
    }

    /// Notifica al editor que un archivo en el proyecto ha cambiado
    pub async fn notify_external_change(&self, project_path: &str) -> Result<()> {
        let url = format!("{}/sorceress/import-asset", self.base_url);
        self.client
            .post(&url)
            .json(&json!({ "project_path": project_path }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Fuerza un rebuild de un atlas específico
    pub async fn rebuild_atlas(&self, atlas_path: &str) -> Result<()> {
        let url = format!("{}/sorceress/rebuild-atlas", self.base_url);
        self.client
            .post(&url)
            .json(&json!({ "atlas_path": atlas_path }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Obtiene información del proyecto actualmente abierto
    pub async fn get_project_info(&self) -> Result<serde_json::Value> {
        let url = format!("{}/sorceress/project-info", self.base_url);
        let info = self.client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        Ok(info)
    }
}
```

---

## 4. Comunicación Bidireccional

### 4.1 Sorceress → Defold (vía MCP tools)

La dirección principal de comunicación: Sorceress (o un agente de IA) invoca MCP tools del
`defold-mcp-server`, que traduce las llamadas en peticiones HTTP al editor de Defold.

```
Sorceress Worker
  │
  │  MCP call: defold_import_spritesheet(source_png, atlas_name, ...)
  ▼
defold-mcp-server (Rust)
  │
  │  1. fs::copy(source_png → project_root/sprites/...)
  │  2. Generar .atlas file (formato text de Defold)
  │  3. POST http://localhost:55555/sorceress/import-asset
  │  4. POST http://localhost:55555/sorceress/rebuild-atlas
  ▼
Defold Editor (Editor Script en Lua)
  │
  │  - editor.external_change("/sprites/player/player.atlas")
  │  - editor.build_resource("/sprites/player/player.atlas")
  ▼
Editor actualizado con nuevo sprite sheet
```

### 4.2 Defold → Sorceress (vía HTTP callback)

El editor de Defold puede notificar a Sorceress cuando el usuario realiza acciones relevantes
(guarda el proyecto, termina un build, añade un asset manualmente). El Editor Script llama
al endpoint HTTP de Sorceress directamente:

```lua
-- editor_script/sorceress_bridge.editor_script

local M = {}

-- URL de la app Sorceress (configurada via game.project o archivo de config)
local SORCERESS_CALLBACK_URL = "http://localhost:9090/defold/event"

local function notify_sorceress(event_type, data)
  local body = json.encode({
    event   = event_type,
    project = editor.get_project_root(),
    data    = data,
    ts      = os.time()
  })
  http.request(
    SORCERESS_CALLBACK_URL,
    "POST",
    function(self, id, response)
      if response.status ~= 200 then
        print("[Sorceress] Callback failed: " .. tostring(response.status))
      end
    end,
    { ["Content-Type"] = "application/json" },
    body
  )
end

function M.on_build_finished(opts)
  notify_sorceress("build_finished", {
    success  = opts.success,
    platform = opts.platform
  })
end

function M.on_resource_added(opts)
  notify_sorceress("resource_added", {
    path = opts.resource_path,
    type = opts.resource_type
  })
end

return M
```

En el lado de Sorceress (Tauri v2 / Rust), existe un endpoint HTTP que recibe estos eventos
y los convierte en `DomainEvent` que fluyen hacia el bounded context correspondiente:

```rust
// src-tauri/src/defold_webhook.rs

use axum::{extract::State, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DefoldEvent {
    pub event: String,
    pub project: String,
    pub data: serde_json::Value,
    pub ts: i64,
}

pub async fn handle_defold_event(
    State(app): State<AppState>,
    Json(event): Json<DefoldEvent>,
) {
    match event.event.as_str() {
        "build_finished" => {
            // Publicar en el event bus interno de Sorceress
            app.event_bus
               .publish(DefoldBuildFinished {
                   project: event.project,
                   success: event.data["success"].as_bool().unwrap_or(false),
               })
               .await;
        }
        "resource_added" => {
            // Notificar al bounded context AssetPipeline que hay un nuevo asset en Defold
            app.asset_pipeline
               .on_defold_resource_added(
                   event.data["path"].as_str().unwrap_or(""),
                   &event.project,
               )
               .await;
        }
        _ => {
            tracing::debug!("Unknown Defold event: {}", event.event);
        }
    }
}
```

---

### 4.3 Flujo Completo — Diagrama ASCII

El flujo de extremo a extremo para generar un sprite desde IA e importarlo a Defold:

```
Usuario en Sorceress UI
  │
  │  "Genera sprite de goblin corriendo, 8 frames"
  ▼
Sorceress Frontend (Svelte/React)
  │
  │  invoke("submit_job", { kind: "SpriteGeneration", ... })
  ▼
Tauri Command Handler (Rust)
  │
  │  JobQueue.submit(SpriteGenerationJob { ... })
  ▼
Job Queue Context
  │
  │  JobSubmitted event  →  Sprite Factory worker (IPC)
  ▼
Sprite Factory Worker (proceso independiente)
  │
  │  1. Llama a Image Engine para generar frames
  │  2. Empaqueta frames en sprite sheet (PNG)
  │  3. SpriteJobCompleted event
  ▼
Job Queue Context
  │
  │  JobCompleted → Asset Management Context
  ▼
Asset Management Context
  │
  │  AssetVersionCreated { asset_id, path: "/tmp/sorceress/goblin_run.png" }
  │
  │  Si el proyecto Defold está configurado como destino de publicación:
  │  dispatch DefoldSyncJob { source_path, target_atlas, ... }
  ▼
Job Queue Context
  │
  │  DefoldSyncJob → DefoldSync Worker (proceso sidecar)
  ▼
DefoldSync Worker
  │
  │  MCP call: defold_import_spritesheet(
  │    source_png:  "/tmp/sorceress/goblin_run.png",
  │    project_dir: "/sprites/enemies/",
  │    atlas_name:  "goblin",
  │    frame_width: 64, frame_height: 64,
  │    animations:  [{ name: "run", from_frame: 0, to_frame: 7, fps: 12 }]
  │  )
  ▼
defold-mcp-server (Rust, localhost:7777)
  │
  │  1. fs::copy(PNG → /home/user/MyGame/sprites/enemies/goblin_run.png)
  │  2. Generar goblin.atlas (formato text Defold)
  │  3. POST http://localhost:55555/sorceress/import-asset
  │  4. POST http://localhost:55555/sorceress/rebuild-atlas
  ▼
Defold Editor (Editor Script Lua)
  │
  │  editor.external_change("/sprites/enemies/goblin.atlas")
  │  editor.build_resource("/sprites/enemies/goblin.atlas")
  ▼
Editor de Defold muestra el nuevo sprite sheet compilado
  │
  │  (Editor Script) → POST http://localhost:9090/defold/event
  │    { event: "build_finished", ... }
  ▼
Sorceress Webhook Handler
  │
  │  DefoldBuildFinished event → UI update
  ▼
Usuario ve confirmación en la UI de Sorceress
  "Sprite goblin importado en Defold correctamente"
```

---

## 5. Integración con Bounded Context `AssetPipeline`

### 5.1 Posición en la Arquitectura DDD

El bounded context `AssetPipeline` (también llamado `Asset Management Context` en el diseño
DDD de Sorceress) es el hub central de todos los assets. Es **downstream** de los engines
(Sprite Factory, Image Engine, etc.) y **upstream** de Publishing y, ahora, de la integración
con Defold.

La integración Defold no es un bounded context independiente: es un **output channel** del
`AssetPipeline` context, similar a como Publishing es otro output channel. Defold recibe
assets finales ya registrados y versionados en Asset Management.

```
┌────────────────────────────────────────────────────────┐
│              ASSET MANAGEMENT CONTEXT                   │
│                                                         │
│  ┌────────────┐    ┌──────────────┐   ┌─────────────┐  │
│  │  Asset     │    │  AssetVersion│   │  Collection │  │
│  │  aggregate │    │  entity      │   │  aggregate  │  │
│  └─────┬──────┘    └──────────────┘   └─────────────┘  │
│        │                                                │
│  Domain Services:                                       │
│  ┌─────────────────────────────────────────────────┐   │
│  │  AssetRegistrar    — registra nuevos assets      │   │
│  │  AssetPublisher    — publica a destinos externos │   │
│  │  DefoldSyncService — sincroniza con Defold       │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  Output Channels (Ports):                               │
│  ┌───────────────┐  ┌────────────────┐                  │
│  │  Publishing   │  │  Defold Sync   │                  │
│  │  Adapter      │  │  Adapter       │                  │
│  └───────────────┘  └───────┬────────┘                  │
└───────────────────────────┬─┼────────────────────────────┘
                            │ │
                            │ │  Job Queue
                            │ │  DefoldSyncJob
                            │ ▼
                     ┌──────────────┐
                     │  Job Queue   │
                     │  Context     │
                     └──────┬───────┘
                            │ dispatch
                            ▼
                     ┌──────────────────────┐
                     │  DefoldSync Worker   │
                     │  (proceso sidecar)   │
                     └──────┬───────────────┘
                            │ MCP calls
                            ▼
                     defold-mcp-server
```

---

### 5.2 Domain Service: `DefoldSyncService`

```rust
// bounded_contexts/asset_pipeline/src/domain/services/defold_sync_service.rs

use crate::domain::{
    Asset, AssetId, AssetVersion, DefoldProjectConfig,
};
use crate::domain::events::DefoldSyncRequested;

pub struct DefoldSyncService;

impl DefoldSyncService {
    /// Decide si un asset debe sincronizarse con Defold y construye el job correspondiente
    pub fn should_sync_to_defold(
        asset: &Asset,
        version: &AssetVersion,
        config: &DefoldProjectConfig,
    ) -> bool {
        // Solo sincronizar si el proyecto Defold está configurado
        if config.project_root.is_none() {
            return false;
        }
        // Solo sincronizar tipos de assets que Defold entiende
        matches!(
            asset.kind,
            AssetKind::Image | AssetKind::SpriteSheet | AssetKind::Audio | AssetKind::Font
        )
    }

    /// Construye el evento de dominio que dispara la sincronización
    pub fn build_sync_event(
        asset: &Asset,
        version: &AssetVersion,
        config: &DefoldProjectConfig,
    ) -> DefoldSyncRequested {
        let target_path = config.map_asset_to_defold_path(asset);

        DefoldSyncRequested {
            asset_id: asset.id.clone(),
            version_id: version.id.clone(),
            source_path: version.storage_path.clone(),
            defold_project_root: config.project_root.clone().unwrap(),
            target_path,
            atlas: config.get_atlas_for_asset(asset),
        }
    }
}
```

---

### 5.3 Eventos de Dominio

Los eventos que fluyen entre el bounded context `AssetPipeline` y la integración Defold:

| Evento                    | Origen               | Consumidor           | Descripción                                    |
|---------------------------|----------------------|----------------------|------------------------------------------------|
| `AssetVersionCreated`     | Asset Management     | Job Queue            | Nuevo asset registrado, puede requerir sync    |
| `DefoldSyncRequested`     | Asset Management     | Job Queue            | Solicitud explícita de sincronización          |
| `DefoldSyncJobDispatched` | Job Queue            | DefoldSync Worker    | Job enviado al worker                          |
| `DefoldSyncCompleted`     | DefoldSync Worker    | Asset Management     | Sincronización exitosa, actualiza Asset        |
| `DefoldSyncFailed`        | DefoldSync Worker    | Job Queue, UI        | Fallo en sincronización, posible retry         |
| `DefoldBuildFinished`     | Defold Editor (webhook) | Asset Management  | Build terminado, los assets están compilados   |

---

### 5.4 Conexión al Job Queue

El Job Queue actúa como mediador entre el Asset Management Context y el DefoldSync Worker.
Introduce un nuevo `WorkerKind::DefoldSync` y un nuevo tipo de job `DefoldSyncJob`.

```rust
// bounded_contexts/job_queue/src/domain/job.rs (extensión)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobKind {
    SpriteGeneration,
    ImageGeneration,
    AudioGeneration,
    VideoProcessing,
    TileGeneration,
    MaterialGeneration,
    AgentExecution,
    DefoldSync,        // NUEVO: sincronización con Defold
    DefoldBuild,       // NUEVO: build del proyecto Defold
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefoldSyncPayload {
    pub asset_id: AssetId,
    pub source_path: PathBuf,
    pub defold_project_root: PathBuf,
    pub target_path: String,           // ruta relativa en el proyecto Defold
    pub atlas: Option<AtlasConfig>,    // si es un sprite sheet
    pub frame_width: Option<u32>,
    pub frame_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasConfig {
    pub name: String,
    pub animations: Vec<AnimationConfig>,
    pub margin: u32,
    pub extrude_borders: u32,
}
```

---

### 5.5 Repository Trait para Configuración Defold

```rust
// bounded_contexts/asset_pipeline/src/domain/repositories.rs (extensión)

#[async_trait]
pub trait DefoldProjectConfigRepository: Send + Sync {
    /// Devuelve la configuración del proyecto Defold asociado al proyecto de Sorceress
    async fn get_config(&self, project_id: &ProjectId)
        -> Result<Option<DefoldProjectConfig>>;

    /// Guarda la configuración (ruta del proyecto, reglas de mapeo, etc.)
    async fn save_config(&self, config: &DefoldProjectConfig)
        -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct DefoldProjectConfig {
    pub project_id: ProjectId,
    pub project_root: Option<PathBuf>,
    pub auto_sync: bool,
    pub path_mapping_rules: Vec<PathMappingRule>,
    pub editor_port: Option<u16>,
}
```

---

## 6. Propuestas de Pensamiento Lateral

### 6.1 Defold como Sistema de Previsualización de Assets

En lugar de usar un visor de sprites propio en Sorceress, se puede usar el editor de Defold
como motor de previsualización en tiempo real. El flujo sería:

- Sorceress importa el asset al proyecto Defold de preview (un proyecto mínimo dedicado).
- El editor de Defold muestra el sprite en acción (con animaciones, iluminación del motor).
- El usuario aprueba el asset desde Defold y la aprobación llega a Sorceress via webhook.

Esto evita duplicar lógica de renderizado y aprovecha el renderer de Defold (que es el
renderer real donde el asset va a vivir).

---

### 6.2 Generación Asistida por Contexto del Proyecto

El `defold_list_assets` tool puede alimentar el contexto del agente de IA con información
del proyecto Defold existente: qué personajes existen, qué animaciones hay, qué paleta de
colores se usa. Con ese contexto, la generación de assets (vía Stable Diffusion o similar)
puede ser coherente con el estilo visual ya establecido en el proyecto.

Flujo: Sorceress llama a `defold_list_assets` antes de generar → pasa los assets existentes
como contexto al modelo generativo → los nuevos assets son estilísticamente consistentes.

---

### 6.3 Hot-Reload de Assets IA durante Playtesting

Durante el desarrollo, el game developer puede estar probando el juego en el editor de
Defold y solicitar a Sorceress variaciones de un asset (diferente color de skin, diferente
expresión facial) sin salir del flujo de playtesting. El ciclo sería:

1. Usuario pausa el play y dice "cambia el color del goblin a azul".
2. Sorceress regenera el sprite con el cambio.
3. Sorceress llama a `defold_import_asset` + `defold_hot_reload`.
4. El editor de Defold recarga el asset sin reiniciar el proyecto.
5. El usuario reanuda el play con el asset actualizado en segundos.

Esto convierte la IA generativa en un colaborador en tiempo real durante el desarrollo.

---

### 6.4 Generación Procedural de Tilemaps desde Defold

La integración puede funcionar en la dirección inversa: el game developer diseña la
estructura de un nivel en Defold (posición de tiles, zonas especiales) y Sorceress lee
esa estructura para generar automáticamente variantes visuales del tileset que encajen
con el layout definido. El `defold_get_selection` puede devolver el tilemap seleccionado,
que Sorceress usa como input para el Tile Forge engine.

---

### 6.5 MCP como Protocolo Universal para Otros Motores

El `defold-mcp-server` establece un patrón replicable. El mismo diseño puede aplicarse a:

- **Godot**: Godot tiene un servidor de LSP y una API de editor remoto via `--remote` flags.
- **Unity**: Unity Remote tiene un protocolo HTTP accesible desde plugins.
- **RPG Maker MZ**: Tiene una API de plugins en JavaScript accesible desde el editor.

Sorceress puede mantener múltiples MCP servers (`godot-mcp-server`, `unity-mcp-server`)
con la misma interfaz de alto nivel, abstrayendo las diferencias de cada editor detrás
del protocolo MCP. El bounded context `AssetPipeline` no necesita saber con qué motor
está hablando — solo despacha `DefoldSyncJob` o `GodotSyncJob` según la configuración.

---

### 6.6 Build CI/CD Orquestado desde Sorceress

El `defold_bundle` tool, combinado con el Job Queue de Sorceress, permite construir un
pipeline CI/CD embebido en la app de escritorio:

1. El usuario finaliza un set de assets y los marca como "release candidate".
2. Sorceress despacha automáticamente una secuencia de jobs:
   - `DefoldSyncJob` × N (sincronizar todos los assets del release).
   - `DefoldBuildJob` × cada plataforma (Linux, macOS, HTML5).
3. Los bundles generados se suben a los destinos configurados (itch.io, GitHub Releases).
4. El Publishing Context de Sorceress gestiona el upload y crea la entrada en el arcade.

Para proyectos indie pequeños, esto elimina la necesidad de configurar GitHub Actions
o Jenkins solo para hacer builds de Defold.

---

## 7. Plan de Implementación en 4 Fases

### Fase 1 — Fundaciones (3-4 semanas)

**Objetivo**: Editor Script funcional + `defold-mcp-server` con tools básicos de assets.

**Tareas:**

1. Crear el repositorio `defold-mcp-server` (crate Rust).
2. Implementar `DefoldClient` con:
   - `PortDiscovery` (leer `.internal/editor-server-port`).
   - Métodos `notify_external_change` y `get_project_info`.
3. Implementar `BobRunner` (wrapper de `tokio::process::Command` para `bob.jar`).
4. Implementar los tools:
   - `defold_get_project_info`
   - `defold_import_asset`
   - `defold_list_assets`
   - `defold_build` (solo, sin extensiones nativas)
5. Crear el Editor Script `sorceress_bridge.editor_script` con:
   - `get_http_server_routes()` (3 endpoints: import, rebuild-atlas, project-info).
   - `notify_sorceress()` (callback a Sorceress).
6. Integración básica: Sorceress puede invocar los tools desde un test CLI.

**Criterio de éxito**: Un PNG generado por Sorceress aparece en el proyecto Defold
abierto en el editor sin intervención manual.

---

### Fase 2 — Sprite Sheets y Atlas (2-3 semanas)

**Objetivo**: Importar sprite sheets completos con `.atlas` generado automáticamente.

**Tareas:**

1. Implementar el generador de archivos `.atlas` en Rust (formato text de Defold).
2. Implementar `defold_import_spritesheet` con soporte de animaciones.
3. Añadir `defold_hot_reload` para recargar atlas sin reiniciar el editor.
4. Conectar el Sprite Factory worker de Sorceress con el `defold-mcp-server`:
   - Añadir `DefoldSync` como `WorkerKind` en el Job Queue.
   - Implementar el `DefoldSync Worker` como proceso sidecar.
5. Implementar el `DefoldSyncService` en el bounded context `AssetPipeline`.
6. Añadir `DefoldProjectConfig` con UI de configuración en Sorceress.

**Criterio de éxito**: El usuario genera un sprite animado en Sorceress y al terminar
el job, el `.atlas` aparece en Defold con las animaciones correctamente definidas.

---

### Fase 3 — Operaciones de Editor y Build (3-4 semanas)

**Objetivo**: Manipulación de escenas y builds automatizados.

**Tareas:**

1. Implementar tools de `scene_ops`:
   - `defold_add_game_object`
   - `defold_update_node_property`
   - `defold_get_selection`
2. Implementar `defold_bundle` con soporte multi-plataforma.
3. Añadir el servidor de webhook en Sorceress para recibir eventos del editor.
4. Implementar los eventos de dominio `DefoldBuildFinished` y `DefoldSyncCompleted`.
5. UI en Sorceress para visualizar el estado de sincronización con Defold
   (qué assets están sincronizados, cuáles están pendientes, errores).
6. Manejar el caso donde el editor de Defold no está abierto (fallback graceful).

**Criterio de éxito**: Sorceress puede lanzar un build de Defold, recibir la
notificación de éxito/fallo, y mostrar el resultado en la UI.

---

### Fase 4 — Madurez y Pipeline CI/CD (4-5 semanas)

**Objetivo**: Pipeline CI/CD embebido y experiencia pulida.

**Tareas:**

1. Implementar `defold_build_with_extensions` (con build server cloud).
2. Implementar el pipeline de "release candidate" en el Publishing Context:
   - Secuencia de jobs: sync → build × plataformas → upload.
3. Soporte para `defold_install_extension`.
4. Implementar el patrón de previsualización en Defold (propuesta 6.1):
   - Proyecto Defold de preview dedicado.
   - Auto-sync de assets a ese proyecto.
5. Tests de integración end-to-end con un proyecto Defold real de prueba.
6. Documentación del Editor Script y guía de instalación para el usuario.
7. Empaquetado del `defold-mcp-server` como sidecar en el installer de Sorceress.

**Criterio de éxito**: El usuario puede hacer el ciclo completo desde Sorceress:
generar assets → sincronizar con Defold → lanzar builds para múltiples plataformas
→ ver el bundle generado — todo desde la UI de Sorceress, sin tocar una terminal.

---

### Resumen de Plazos y Dependencias

```
Fase 1   ──────────────────────►  Semanas 1-4
  DefoldClient + Editor Script básico + tools asset_ops básicos

Fase 2   ──────────────────────►  Semanas 5-7
  (depende de Fase 1)
  Sprite sheets + atlas + DefoldSync Worker + Job Queue integration

Fase 3   ──────────────────────►  Semanas 8-11
  (depende de Fase 2)
  scene_ops + builds + webhook + UI de estado

Fase 4   ──────────────────────►  Semanas 12-16
  (depende de Fase 3)
  CI/CD pipeline + extensiones nativas + preview + packaging
```

**Nota sobre dependencias externas**: Las Fases 1 y 2 no requieren credenciales del
build server de Defold ni acceso a extensiones nativas. Pueden desarrollarse con una
instalación estándar de Defold. Las Fases 3 y 4 requieren una cuenta Defold activa
para probar el build server cloud.

---

*Documento generado: 2026-04-14*
*Próxima revisión sugerida: después de completar la Fase 1 (proof of concept)*
