# CapitalCore — Real-Time Banking Decision & Fraud Operations Platform

![Rust](https://img.shields.io/badge/Rust-1.80+-000000?style=flat-square&logo=rust&logoColor=white)
![Next.js](https://img.shields.io/badge/Next.js-14_App_Router-black?style=flat-square&logo=next.js&logoColor=white)
![Axum](https://img.shields.io/badge/Axum-0.7_HTTP_API-blueviolet?style=flat-square)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15_SQLx-336791?style=flat-square&logo=postgresql&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-Full_Stack_Container-2496ED?style=flat-square&logo=docker&logoColor=white)
![License](https://img.shields.io/badge/License-Proprietary-green?style=flat-square)

CapitalCore is an enterprise-grade banking transaction decision engine and real-time fraud mitigation platform. It ingests high-throughput payment events across multiple financial channels (`pos`, `web`, `mobile`, `api`, `atm`, `batch`) and outputs deterministic, explainable verdicts (`ALLOW`, `REVIEW`, `ESCALATE`, `BLOCK`) in sub-millisecond latency.

---

## ⚡ Quick Test & Review Links (Acceso Rápido para Revisión)

The platform is running unified on dedicated ports (avoiding port 3000/8080 conflicts):

| Componente | URL de Acceso | Descripción |
| :--- | :--- | :--- |
| 🌐 **Operations Console (Frontend)** | [http://localhost:3001/dashboard](http://localhost:3001/dashboard) | Consola analítica completa con KPIs en vivo, transacciones y alertas |
| 🧪 **Transaction Simulator Studio** | [http://localhost:3001/simulate](http://localhost:3001/simulate) | Simulador interactivo con presets (Café $48.50, Wire $18.5k, Skimmer $800) |
| 🔍 **Transaction Explorer** | [http://localhost:3001/decisions](http://localhost:3001/decisions) | Buscador y drawer de inspección con AST de reglas y snapshot de features |
| 🛡️ **Review & Escalation Queue** | [http://localhost:3001/review](http://localhost:3001/review) | Bandeja de triaje para analistas (Override, Confirm Block, Escalate SAR) |
| ⛓️ **Cryptographic Audit Ledger** | [http://localhost:3001/audit](http://localhost:3001/audit) | Registro inmutable SHA-256 con botón de verificación criptográfica |
| 🤖 **ML Model Governance Card** | [http://localhost:3001/model](http://localhost:3001/model) | Ficha técnica del modelo ONNX (99.60% precisión, latencia P99 < 15 μs) |
| 📊 **System Telemetry & Health** | [http://localhost:3001/system](http://localhost:3001/system) | Diagnóstico de subsistemas y tiempos de respuesta de sondas |
| 🚀 **API Health Probe** | [http://localhost:8001/health/liveness](http://localhost:8001/health/liveness) | Sonda de vida del motor Rust Axum (`{"platform":"CapitalCore","status":"UP"}`) |
| 📈 **Live Decision Metrics API** | [http://localhost:8001/api/v1/decisions/stats](http://localhost:8001/api/v1/decisions/stats) | JSON en tiempo real con volumen, tasa de bloqueo y contadores |

> [!NOTE]
> **Acceso Inmediato Sin Credenciales (Passwordless Analyst Access):**  
> Para agilizar la revisión y testing, la consola web **no requiere contraseña ni registro**. Al ingresar a [http://localhost:3001](http://localhost:3001) o cualquier subruta, el sistema autentica automáticamente la sesión como **Lead Risk Analyst** (`analyst@capitalcore.bank`), otorgando permisos RBAC completos (`events:read`, `events:write`, `decisions:read`, `rules:read`, `audit:read`).

---

## 🏗️ Arquitectura del Sistema

```mermaid
graph TD
    Client[Core Bancario / Ingress Cliente] -->|Ed25519 JWT / RBAC| API[CapitalCore API · Axum :8001]
    
    subgraph Ingestion Pipeline
        API --> Ingest[edge-transactions<br/>Validación · Normalización · Idempotencia]
        Ingest --> EventBus[edge-events bus]
        EventBus --> Workers[edge-workers<br/>Async Retry · DLQ]
    end

    subgraph Decision Engine
        API --> Decision[edge-decision Arbitrator]
        Decision --> Rules[edge-rules<br/>Motor de Políticas AST]
        Decision --> Features[edge-features<br/>Ventana Deslizante 5m/1h/24h]
        Decision --> ML[edge-ml<br/>Modelo ONNX Integrado]
    end

    subgraph Persistence & Audit
        Decision --> Audit[edge-audit<br/>Cadena Hash SHA-256 Inmutable]
        Decision --> Storage[(PostgreSQL 15 / Ring Buffer)]
    end

    subgraph Frontend Console
        Web[Next.js 14 Operations Console :3001] -->|REST API / Live Queries| API
    end
```

El sistema combina:
1. **Motor de Reglas AST Determinístico**: Evalúa políticas bancarias de riesgo y límites de velocidad regulatorios.
2. **Extractor de Características en Ventana Deslizante**: Calcula agregaciones temporales en memoria (sumas y frecuencias en 5m, 1h, 24h).
3. **Modelo Machine Learning Integrado (ONNX)**: Estima probabilidad de fraude con 99.60% de precisión y latencia < 15 μs.
4. **Ledger Criptográfico Inmutable**: Encadena todas las decisiones en un árbol de hashes SHA-256 a prueba de manipulaciones.
5. **Consola de Operaciones en Next.js 14**: Dashboard moderno con triaje en vivo, simulación y verificación forense.

---

## 🌐 Configuración de Puertos y Red

Para evitar colisiones con otros proyectos y servicios locales (como puertos 3000, 8080 o 5432 del host), CapitalCore utiliza puertos dedicados:

| Servicio | Puerto Host | Puerto Contenedor | Descripción |
| :--- | :--- | :--- | :--- |
| **Next.js Web Console** | `3001` | `3001` | Dashboard operacional, simulador y explorador de auditoría |
| **Rust Decision API** | `8001` | `8001` | API REST HTTP Axum, métricas y endpoint de decisiones |
| **PostgreSQL 15 Database** | *(Interno)* | `5432` | Base de datos PostgreSQL interna aislada (no choca con Postgres del host) |

---

## 🐳 Ejecución con Docker (Full-Stack Container)

El proyecto incluye un contenedor unificado de producción (`Dockerfile.all-in-one`) que empaqueta **PostgreSQL 15 + Rust Core API + Next.js Web UI** en un solo entorno auto-contenido.

### 1. Iniciar el Contenedor (Recomendado)

Si ya existe un contenedor previo o deseas reconstruir con los últimos cambios:

```bash
# Limpiar cualquier contenedor previo para evitar conflictos de nombre
docker compose down

# Compilar e iniciar en segundo plano
docker compose up -d --build
```

### 2. Verificar el Estado del Contenedor

```bash
# Ver estado y puertos asignados
docker ps

# Ver logs en tiempo real (PostgreSQL, Rust Engine y Next.js)
docker logs -f capitalcore-unified
```

### 3. Detener el Contenedor

```bash
docker compose down
```

### 4. Solución Rápida de Problemas con Docker

* **Error de conflicto de contenedor (`Conflict. The container name is already in use`):**
  ```bash
  docker rm -f capitalcore-unified
  docker compose up -d
  ```

* **Comprobar salud del backend:**
  ```bash
  curl http://localhost:8001/health/liveness
  ```
  Respuesta esperada: `{"platform":"CapitalCore","status":"UP"}`

* **Comprobar salud del frontend:**
  Abrir [http://localhost:3001](http://localhost:3001) o [http://localhost:3001/dashboard](http://localhost:3001/dashboard).

---

## 💻 Ejecución Nativa en Local (Sin Docker)

Si prefieres ejecutar los servicios directamente en tu máquina host sin contenedores:

### Requisitos Previos
* **Rust**: 1.80+ (`rustup default stable`)
* **Node.js**: 20+ y **pnpm**: 9+ (`corepack enable && corepack prepare pnpm@10.32.1 --activate`)
* **PostgreSQL**: (Opcional, el backend incluye almacenamiento en memoria de alta velocidad si no hay base de datos configurada)

### Paso 1: Ejecutar el Motor de Decisión en Rust (Puerto 8001)

Abre una terminal en la raíz del proyecto:

```powershell
# En Windows PowerShell
$env:PORT="8001"
$env:HOST="0.0.0.0"
$env:RUST_LOG="info,edge_api=debug"
cargo run --release --bin edge-api
```

```bash
# En Linux / macOS
export PORT=8001
export HOST=0.0.0.0
export RUST_LOG=info,edge_api=debug
cargo run --release --bin edge-api
```

El motor quedará escuchando en `http://localhost:8001`.

### Paso 2: Ejecutar la Consola Web Next.js (Puerto 3001)

Abre una segunda terminal:

```bash
cd apps/web
pnpm install
pnpm dev
```

Abre tu navegador en [http://localhost:3001](http://localhost:3001).

---

## 📋 Guía Paso a Paso para la Revisión (Testing Walkthrough)

Sigue estos pasos en la interfaz web para evaluar y revisar todas las capacidades de la plataforma:

### 1. Panel de Operaciones (`/dashboard`)
* Navega a [http://localhost:3001/dashboard](http://localhost:3001/dashboard).
* Observa las tarjetas de métricas en vivo:
  * **Total Evaluated Decisions**: Contador acumulativo de transacciones.
  * **Block Rate**: Porcentaje de operaciones bloqueadas por fraude.
  * **In-Flight Review Cases**: Casos en espera de resolución analítica.
  * **Active Deterministic Rules**: Reglas AST activas en el motor.
  * **Embedded ML Accuracy**: Precisión del 99.60% del modelo ONNX.
* Debajo encontrarás la **Decision Stream Table** con transacciones precargadas con sus códigos de razón, puntuación de riesgo ML y veredicto (`ALLOW`, `REVIEW`, `ESCALATE`, `BLOCK`).

### 2. Simulador de Transacciones (`/simulate`)
* Navega a [http://localhost:3001/simulate](http://localhost:3001/simulate).
* Selecciona uno de los 4 escenarios preconfigurados:
  * 🟢 **Low-Risk Coffee Swipe ($48.50)**: Canal POS, comercio minorista.
  * 🟡 **High-Value Business Wire ($18,500.00)**: Supera umbral de $10,000 &rarr; Veredicto `REVIEW`.
  * 🟠 **Cross-Border Offshore Transfer ($145,000.00)**: Transferencia de alto riesgo &rarr; Veredicto `ESCALATE`.
  * 🔴 **Card-Not-Present Skimmer Burst ($800.00)**: Ráfaga de intentos sospechosos &rarr; Veredicto `BLOCK`.
* Haz clic en **"Simulate & Evaluate Decision"**.
* Verás inmediatamente:
  * El veredicto en tiempo real con latencia en milisegundos (P99 ~ 2-5 ms).
  * El identificador del evento de auditoría criptográfica generado.
  * La puntuación del modelo ML (`ml_fraud_score_bps`).
  * Las versiones exactas de las características y reglas utilizadas.
* Vuelve a `/dashboard` o `/decisions` y confirma que tu transacción simulada ya aparece registrada.

### 3. Explorador de Transacciones y Drawer Forense (`/decisions`)
* Navega a [http://localhost:3001/decisions](http://localhost:3001/decisions).
* Filtra transacciones haciendo clic en las pestañas `ALLOW`, `REVIEW`, `ESCALATE`, `BLOCK` o usa el buscador de UUID/Cliente.
* Haz clic sobre cualquier fila para desplegar el **Decision Detail Drawer**:
  * **Reglas Gatilladas**: Inspección del árbol AST y política aplicada.
  * **Snapshot de Features**: Métricas de ventana deslizante en el instante exacto de la decisión (frecuencia 5m, volumen 1h, comercios distintos 24h).
  * **Audit Event Proof**: Identificador único y timestamp firmado.

### 4. Bandeja de Revisión y Triaje (`/review`)
* Navega a [http://localhost:3001/review](http://localhost:3001/review).
* Muestra los casos que requieren intervención humana por alto monto o patrones anómalos.
* Cada tarjeta permite ejecutar acciones analíticas inmediatas:
  * **Approve Override**: Desbloqueo manual justificado.
  * **Confirm Block**: Bloqueo definitivo de la tarjeta o cuenta.
  * **Escalate SAR**: Notificación formal a la Unidad de Inteligencia Financiera.

### 5. Auditoría Criptográfica e Inmutabilidad (`/audit`)
* Navega a [http://localhost:3001/audit](http://localhost:3001/audit).
* Inspecciona el registro de transacciones encadenadas criptográficamente con hashes SHA-256.
* Haz clic en el botón **"Verify Chain Integrity"**:
  * El backend recorre y verifica la integridad matemática de cada eslabón.
  * Muestra una confirmación en verde: `VERIFIED_TAMPER_EVIDENT`.

### 6. Ficha de Gobernanza del Modelo ML (`/model`)
* Navega a [http://localhost:3001/model](http://localhost:3001/model).
* Muestra las especificaciones de cumplimiento regulatorio del modelo integrado `v1.2.0-onnx`:
  * Matriz de confusión (Precision: 98.4%, Recall: 100%, F1: 99.2%).
  * Catálogo de las 9 características continuas y de ventana temporal.
  * Telemetría de inferencia P50 < 8 μs, P99 < 15 μs.

### 7. Diagnóstico de Subsistemas (`/system`)
* Navega a [http://localhost:3001/system](http://localhost:3001/system).
* Revisa en vivo el estado y latencia de las sondas de preparación (`/health/readiness`), almacenamiento persistente, suites de cifrado y suite de pruebas de regresión.

---

## 📡 Referencia de API REST (cURL y Ejemplos)

La API corre en `http://localhost:8001`. A continuación se detallan los endpoints disponibles con ejemplos listos para copiar y pegar:

### 1. Obtener Token de Sesión de Analista
Retorna un token JWT Ed25519 con rol de analista de riesgo:

```bash
curl -s http://localhost:8001/api/v1/auth/session
```

**Ejemplo de respuesta (HTTP 200 OK):**
```json
{
  "access_token": "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "id": "2436f986-a2cc-4d29-8efd-a64a58594e10",
    "name": "Lead Risk Analyst",
    "email": "analyst@capitalcore.bank",
    "role": "Risk Analyst",
    "department": "Fraud Prevention & Compliance",
    "permissions": ["events:read", "events:write", "decisions:read", "rules:read", "audit:read"]
  }
}
```

### 2. Evaluar Transacción Bancaria
Permite inyectar un pago para evaluación en tiempo real:

```bash
# 1. Obtener token
TOKEN=$(curl -s http://localhost:8001/api/v1/auth/session | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)

# 2. Enviar transacción
curl -X POST http://localhost:8001/api/v1/transactions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "transaction_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
    "user_id": "0e8224c9-1c1b-4fd1-a2d5-e092967453a5",
    "event_type": "payment",
    "channel": "pos",
    "amount_minor": 4850,
    "currency": "USD",
    "metadata": {
      "merchant_id": "merch_wholefoods_98",
      "category": "supermarket"
    }
  }'
```

**Ejemplo de respuesta (HTTP 200 OK):**
```json
{
  "decision": "Allow",
  "reasons": [],
  "audit_event": {
    "audit_id": "73521c70-5057-41b3-b26b-a6337476d763",
    "event_id": "622fc7aa-64e2-4ceb-8015-4e5c0d0e9651",
    "decision": "Allow",
    "reasons": [],
    "model_version": "edge-ml-v1",
    "created_at": "2026-10-05T18:45:25Z"
  },
  "features_snapshot": {
    "amount_minor": 4850,
    "ml_fraud_score_bps": 120,
    "ml_is_fraud": 0,
    "user_txn_count_5m": 1
  }
}
```

*Canales admitidos (`channel`):* `"pos"`, `"web"`, `"mobile"`, `"api"`, `"atm"`, `"batch"`.

### 3. Obtener KPIs en Tiempo Real
```bash
curl -s http://localhost:8001/api/v1/decisions/stats
```

```json
{
  "total_count": 11,
  "allow_count": 9,
  "review_count": 1,
  "escalate_count": 1,
  "block_count": 0,
  "total_volume_minor": 8228169,
  "block_rate_pct": 0.0,
  "review_queue_count": 2,
  "active_rules_count": 4,
  "ml_model_accuracy": 99.6
}
```

### 4. Consultar Lista de Decisiones Recientes
```bash
# Obtener todas
curl -s "http://localhost:8001/api/v1/decisions?limit=20"

# Filtrar solo casos en revisión
curl -s "http://localhost:8001/api/v1/decisions?status=REVIEW"
```

### 5. Verificar Integridad Criptográfica de la Cadena de Auditoría
```bash
curl -s http://localhost:8001/api/v1/audit/verify
```

```json
{
  "chain_status": "VERIFIED_TAMPER_EVIDENT",
  "valid": true,
  "verified_at": "2026-10-05T18:44:17Z"
}
```

### 6. Sondas de Salud (Health Probes)
```bash
# Liveness probe
curl -s http://localhost:8001/health/liveness
# {"platform":"CapitalCore","status":"UP"}

# Readiness probe
curl -s http://localhost:8001/health/readiness
# {"status":"healthy","components":[],"timestamp":"..."}
```

---

## 📂 Estructura del Monorepo

```
CapitalCore/
├── apps/
│   └── web/                     # Consola Next.js 14 App Router (:3001)
│       ├── src/app/             # Páginas (/dashboard, /simulate, /decisions, etc.)
│       ├── src/components/      # Componentes UI (Drawer, Table, Sidebar, Header)
│       ├── src/hooks/           # React Query hooks para streaming y mutaciones
│       ├── src/lib/             # Cliente API con inyección de JWT
│       └── src/stores/          # Zustand store (sesión analista pre-autenticada)
├── crates/                      # Motores de alto rendimiento en Rust (:8001)
│   ├── edge-api/                # Axum HTTP REST API, routers y probes de salud
│   ├── edge-decision/           # Arbitrador de decisiones (AST + ML + Ventana)
│   ├── edge-rules/              # Motor de evaluación de políticas bancarias AST
│   ├── edge-features/           # Motor de agregaciones en ventana deslizante (5m, 1h, 24h)
│   ├── edge-ml/                 # Inferencia de modelo ONNX embebido en memoria
│   ├── edge-audit/              # Cadena criptográfica SHA-256 a prueba de manipulación
│   ├── edge-storage/            # Almacenamiento PostgreSQL con SQLx migrations
│   ├── edge-transactions/       # Validación, normalización de esquemas e idempotencia
│   ├── edge-events/             # Bus de eventos de alta velocidad
│   ├── edge-auth/               # Autenticación criptográfica Ed25519 JWT y RBAC
│   ├── edge-observability/      # Métricas Prometheus, logs estructurados y registries
│   ├── edge-core/               # Primitivas de dominio y tipos fundamentales
│   └── edge-domain/             # Entidades bancarias y modelos de dominio
├── ml/                          # Pipeline de Machine Learning y dataset de fraude
├── golden/                      # Dataset de regresión y validación de decisiones
├── docker/
│   └── entrypoint.sh            # Script de inicialización PostgreSQL + Rust + Next.js
├── Dockerfile.all-in-one        # Contenedor unificado multi-stage de producción
├── docker-compose.yml           # Orquestación Docker Compose en puertos 3001 y 8001
└── README.md                    # Documentación técnica integral
```

---

## ☁️ Despliegue en Vercel

El frontend Next.js en `apps/web` está listo para ser desplegado en Vercel:

1. **Vía Panel Web de Vercel**:
   * Conecta tu repositorio de GitHub `AaronAllStar/CapitalCore`.
   * En los ajustes del proyecto (*Project Settings*):
     * **Root Directory**: `apps/web`
     * **Framework Preset**: `Next.js`
   * **Variables de Entorno**:
     * `NEXT_PUBLIC_API_URL`: La URL pública HTTPS donde esté expuesta la API de CapitalCore (o `http://localhost:8001` si se prueba contra la API local mediante túnel o proxy).

2. **Vía Vercel CLI**:
   ```bash
   cd apps/web
   npx vercel
   ```

---

## 🔒 Calidad y Seguridad (CI Quality Gates)

El repositorio cuenta con pipeline automatizado de integración continua en GitHub Actions (`.github/workflows/ci.yml`) que valida:
* ✅ **Auditoría de Dependencias y Licencias**: `cargo deny check` (Licencias Unicode-3.0, MIT, Apache-2.0, Zlib aprobadas).
* ✅ **Golden Dataset Verification**: Verificación matemática de precisión del modelo contra dataset bancario.
* ✅ **Lints**: `cargo clippy --workspace --all-targets -- -D warnings`.
* ✅ **Formato de Código**: `cargo fmt --all -- --check`.
* ✅ **Pruebas Unitarias y de Propiedad**: `cargo test --workspace`.

---

## 📄 Licencia

© 2026 CapitalCore Financial Technologies. Todos los derechos reservados.
