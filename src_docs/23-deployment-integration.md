# Deployment & Integration

Deploy TypF in production environments and integrate with existing systems.

## Production Deployment

### Server Architecture

```rust
// Production service with TYPF
use std::sync::Arc;
use tokio::sync::Semaphore;

struct TextRenderService {
    pipeline: Arc<Pipeline>,
    semaphore: Arc<Semaphore>, // Limit concurrent renders
}

impl TextRenderService {
    fn new() -> Result<Self> {
        let pipeline = Arc::new(
            PipelineBuilder::new()
                .shaper(ShaperBackend::HarfBuzz)
                .renderer(RendererBackend::Skia)
                .enable_font_cache(true)
                .cache_size(500 * 1024 * 1024) // 500MB
                .build()?
        );
        
        Ok(Self {
            pipeline,
            semaphore: Arc::new(Semaphore::new(100)), // Max 100 concurrent
        })
    }
    
    async fn render_text(&self, request: RenderRequest) -> Result<Vec<u8>> {
        let _permit = self.semaphore.acquire().await?;
        
        tokio::task::spawn_blocking(move || {
            let result = self.pipeline.render_text(&request.text, &request.font, &request.options)?;
            result.as_png()
        }).await?
    }
}
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release --features "shaping-harfbuzz,render-skia"

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libfontconfig1 \
    libharfbuzz0b \
    libskia0 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/typf-cli /usr/local/bin/
COPY --from=builder /app/examples/fonts/ /app/fonts/

WORKDIR /app
EXPOSE 8080
CMD ["typf-cli", "serve", "--host", "0.0.0.0", "--port", "8080"]
```

### Kubernetes Configuration

```yaml
# k8s-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: typf-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: typf-service
  template:
    metadata:
      labels:
        app: typf-service
    spec:
      containers:
      - name: typf
        image: typf:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        env:
        - name: TYPF_FONT_CACHE_SIZE
          value: "500MB"
        - name: TYPF_MAX_CONCURRENT
          value: "100"
        volumeMounts:
        - name: fonts
          mountPath: /app/fonts
      volumes:
      - name: fonts
        configMap:
          name: font-files
```

## Web Service Integration

### HTTP API Server

```rust
use axum::{extract::Query, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RenderQuery {
    text: String,
    font: String,
    size: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    format: Option<String>,
}

#[derive(Serialize)]
struct RenderResponse {
    success: bool,
    data: Option<String>, // Base64 encoded image
    error: Option<String>,
}

async fn render_endpoint(
    Query(query): Query<RenderQuery>,
    service: axum::extract::State<Arc<TextRenderService>>
) -> Result<Json<RenderResponse>, StatusCode> {
    let font = service.load_font(&query.font).await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let options = RenderOptions {
        font_size: query.size.unwrap_or(16),
        width: query.width.unwrap_or(800),
        height: query.height.unwrap_or(600),
        ..Default::default()
    };
    
    let request = RenderRequest {
        text: query.text,
        font,
        options,
    };
    
    match service.render_text(request).await {
        Ok(png_data) => {
            let base64 = base64::encode(png_data);
            Ok(Json(RenderResponse {
                success: true,
                data: Some(format!("data:image/png;base64,{}", base64)),
                error: None,
            }))
        }
        Err(e) => Ok(Json(RenderResponse {
            success: false,
            data: None,
            error: Some(e.to_string()),
        }))
    }
}

#[tokio::main]
async fn main() {
    let service = Arc::new(TextRenderService::new().unwrap());
    
    let app = axum::Router::new()
        .route("/render", axum::routing::get(render_endpoint))
        .with_state(service)
        .route("/health", axum::routing::get(health_check));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### FastAPI Integration (Python)

```python
from fastapi import FastAPI, HTTPException, Query
from fastapi.responses import Response
import base64
import typf

app = FastAPI(title="TypF Render Service")

# Initialize TypF once
typf_instance = typf.Typf()
typf_instance.load_font("Roboto-Regular.ttf")

@app.get("/render")
async def render_text(
    text: str,
    font: str = "Roboto-Regular.ttf",
    size: int = 16,
    width: int = 800,
    height: int = 600,
    format: str = "png"
):
    try:
        result = typf_instance.render_text(
            text=text,
            font_path=font,
            font_size=size,
            width=width,
            height=height
        )
        
        if format == "png":
            png_data = result.as_png()
            return Response(content=png_data, media_type="image/png")
        elif format == "svg":
            svg_data = result.as_svg().decode('utf-8')
            return Response(content=svg_data, media_type="image/svg+xml")
        elif format == "base64":
            png_data = result.as_png()
            base64_data = base64.b64encode(png_data).decode()
            return {"success": True, "data": base64_data}
        else:
            raise HTTPException(status_code=400, detail="Unsupported format")
            
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/health")
async def health_check():
    return {"status": "healthy", "fonts_loaded": len(typf_instance.list_fonts())}
```

## Database Integration

### Font Storage

```sql
-- Font management schema
CREATE TABLE fonts (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    family VARCHAR(255) NOT NULL,
    style VARCHAR(100) NOT NULL,
    file_data BYTEA NOT NULL,
    file_hash VARCHAR(64) NOT NULL UNIQUE,
    file_size INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    last_used TIMESTAMP
);

-- Render job tracking
CREATE TABLE render_jobs (
    id SERIAL PRIMARY KEY,
    text TEXT NOT NULL,
    font_id INTEGER REFERENCES fonts(id),
    options JSONB NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    result_data BYTEA,
    created_at TIMESTAMP DEFAULT NOW(),
    completed_at TIMESTAMP,
    error_message TEXT
);

-- Indexes for performance
CREATE INDEX idx_fonts_family ON fonts(family);
CREATE INDEX idx_fonts_last_used ON fonts(last_used);
CREATE INDEX idx_render_jobs_status ON render_jobs(status);
CREATE INDEX idx_render_jobs_created ON render_jobs(created_at);
```

### Caching Layer

```rust
use redis::{Commands, RedisError};

struct RedisFontCache {
    client: redis::Client,
    ttl: u64, // Time to live in seconds
}

impl RedisFontCache {
    fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client, ttl: 3600 })
    }
    
    fn cache_render(&self, key: &str, data: &[u8]) -> Result<()> {
        let mut con = self.client.get_connection()?;
        con.set_ex(key, data, self.ttl)?;
        Ok(())
    }
    
    fn get_cached_render(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut con = self.client.get_connection()?;
        let result: Option<Vec<u8>> = con.get(key)?;
        Ok(result)
    }
    
    fn cache_key(&self, text: &str, font_id: u32, options: &RenderOptions) -> String {
        format!("render:{}:{}:{}", 
                hash_string(text), 
                font_id, 
                hash_options(options))
    }
}
```

## Message Queue Integration

### Background Processing

```rust
use lapin::{Channel, Connection, ConnectionProperties, Queue};
use tokio-executor-trait::TokioExecutorTrait;

struct QueueWorker {
    channel: Channel,
    pipeline: Arc<Pipeline>,
}

impl QueueWorker {
    async fn new(amqp_url: &str, pipeline: Arc<Pipeline>) -> Result<Self> {
        let conn = Connection::connect(amqp_url, ConnectionProperties::default().with_executor(Tokio)).await?;
        let channel = conn.create_channel().await?;
        
        let queue = channel.queue_declare(
            "render_jobs",
            lapin::options::QueueDeclareOptions::default(),
            lapin::types::FieldTable::default()
        ).await?;
        
        Ok(Self { channel, pipeline })
    }
    
    async fn start_worker(&self) -> Result<()> {
        let consumer = self.channel
            .basic_consumer("render_jobs", "worker", lapin::options::BasicConsumeOptions::default(), lapin::types::FieldTable::default())
            .await?;
        
        consumer.set_delegate(move |delivery| async move {
            if let Some(delivery) = delivery {
                let job: RenderJob = serde_json::from_slice(&delivery.data)?;
                
                match self.process_job(job).await {
                    Ok(_) => {
                        delivery.ack(lapin::options::BasicAckOptions::default()).await?;
                    }
                    Err(e) => {
                        log::error!("Job failed: {}", e);
                        delivery.nack(lapin::options::BasicNackOptions::default()).await?;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    async fn process_job(&self, job: RenderJob) -> Result<()> {
        let font = self.load_font_by_id(job.font_id).await?;
        let result = self.pipeline.render_text(&job.text, &font, &job.options)?;
        
        // Store result or send to callback
        self.store_result(job.id, &result).await?;
        Ok(())
    }
}
```

### Job Producer

```python
import pika
import json
import uuid

class RenderJobProducer:
    def __init__(self, amqp_url: str):
        self.connection = pika.BlockingConnection(pika.URLParameters(amqp_url))
        self.channel = self.connection.channel()
        self.channel.queue_declare(queue='render_jobs')
    
    def submit_render_job(self, text: str, font_name: str, options: dict, callback_url: str = None):
        job = {
            'id': str(uuid.uuid4()),
            'text': text,
            'font_name': font_name,
            'options': options,
            'callback_url': callback_url,
            'submitted_at': datetime.utcnow().isoformat()
        }
        
        self.channel.basic_publish(
            exchange='',
            routing_key='render_jobs',
            body=json.dumps(job),
            properties=pika.BasicProperties(
                delivery_mode=2,  # make message persistent
            )
        )
        
        return job['id']

# Usage example
producer = RenderJobProducer('amqp://localhost')
job_id = producer.submit_render_job(
    text="Hello, World!",
    font_name="Roboto-Regular",
    options={'size': 24, 'width': 400, 'height': 100},
    callback_url="https://api.example.com/render-complete"
)
```

## Monitoring & Observability

### Metrics Collection

```rust
use prometheus::{Counter, Histogram, Gauge, IntGauge};

struct RenderMetrics {
    // Request metrics
    render_requests_total: Counter,
    render_duration: Histogram,
    
    // System metrics
    active_fonts: IntGauge,
    memory_usage: Gauge,
    cache_hit_rate: Gauge,
    
    // Error metrics
    render_errors_total: Counter,
    font_load_errors_total: Counter,
}

impl RenderMetrics {
    fn new() -> Self {
        Self {
            render_requests_total: Counter::new("typf_render_requests_total", "Total render requests").unwrap(),
            render_duration: Histogram::new("typf_render_duration_seconds", "Render duration").unwrap(),
            active_fonts: IntGauge::new("typf_active_fonts", "Number of active fonts").unwrap(),
            memory_usage: Gauge::new("typf_memory_usage_bytes", "Memory usage in bytes").unwrap(),
            cache_hit_rate: Gauge::new("typf_cache_hit_rate", "Cache hit rate").unwrap(),
            render_errors_total: Counter::new("typf_render_errors_total", "Total render errors").unwrap(),
            font_load_errors_total: Counter::new("typf_font_load_errors_total", "Total font load errors").unwrap(),
        }
    }
    
    fn record_render(&self, duration: Duration, success: bool) {
        self.render_requests_total.inc();
        self.render_duration.observe(duration.as_secs_f64());
        
        if !success {
            self.render_errors_total.inc();
        }
    }
    
    fn update_system_metrics(&self, pipeline: &Pipeline) {
        self.active_fonts.set(pipeline.font_count() as i64);
        self.memory_usage.set(pipeline.memory_usage() as f64);
        self.cache_hit_rate.set(pipeline.cache_hit_rate());
    }
}
```

### Health Checks

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HealthStatus {
    status: String,
    version: String,
    uptime_seconds: u64,
    font_cache_size: usize,
    memory_usage_mb: f64,
    last_render_time: Option<f64>,
    error_rate: f64,
}

impl HealthStatus {
    fn is_healthy(&self) -> bool {
        self.status == "healthy" 
            && self.memory_usage_mb < 1000.0 // 1GB limit
            && self.error_rate < 0.05 // 5% error rate limit
    }
}

async fn health_check(
    metrics: axum::extract::State<Arc<RenderMetrics>>,
    pipeline: axum::extract::State<Arc<Pipeline>>
) -> impl axum::response::IntoResponse {
    let status = HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: get_uptime_seconds(),
        font_cache_size: pipeline.font_count(),
        memory_usage_mb: pipeline.memory_usage() as f64 / (1024.0 * 1024.0),
        last_render_time: metrics.last_render_duration(),
        error_rate: metrics.error_rate(),
    };
    
    let status_code = if status.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    
    (status_code, Json(status))
}
```

## Scaling Strategies

### Horizontal Scaling

```yaml
# Docker Compose for scaling
version: '3.8'
services:
  typf-service:
    image: typf:latest
    replicas: 3
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '1.0'
    environment:
      - TYPF_REDIS_URL=redis://redis:6379
      - TYPF_FONT_CACHE_SIZE=200MB
    depends_on:
      - redis
      - postgres
  
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - typf-service
  
  redis:
    image: redis:alpine
    volumes:
      - redis_data:/data
  
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: typf
      POSTGRES_USER: typf
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
```

### Load Balancing

```nginx
# nginx.conf
upstream typf_backend {
    least_conn;
    server typf-service-1:8080 max_fails=3 fail_timeout=30s;
    server typf-service-2:8080 max_fails=3 fail_timeout=30s;
    server typf-service-3:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    
    location /render {
        proxy_pass http://typf_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_connect_timeout 5s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
    
    location /health {
        proxy_pass http://typf_backend;
        access_log off;
    }
}
```

## Security Considerations

### Authentication

```rust
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request.headers().get("Authorization");
    
    match auth_header {
        Some(header) => {
            if validate_api_token(header.to_str().unwrap()) {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED)
    }
}

fn validate_api_token(token: &str) -> bool {
    // Validate against database or config
    token.starts_with("typf_") && token.len() == 32
}
```

### Input Validation

```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
struct ValidatedRenderRequest {
    #[validate(length(min = 1, max = 10000))]
    text: String,
    
    #[validate(length(min = 1, max = 255))]
    font: String,
    
    #[validate(range(min = 8, max = 1024))]
    size: u32,
    
    #[validate(range(min = 64, max = 4096))]
    width: u32,
    
    #[validate(range(min = 64, max = 4096))]
    height: u32,
    
    #[validate(regex(path = "^[a-zA-Z0-9_-]+$"))]
    format: String,
}

async function validated_render_endpoint(Json(request): Json<ValidatedRenderRequest>) -> Result<Response, StatusCode> {
    if let Err(errors) = request.validate() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Process validated request
    process_render(request).await
}
```

## Configuration Management

### Environment Configuration

```toml
# production.toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[cache]
font_cache_size = "500MB"
render_cache_size = "1GB"
redis_url = "redis://localhost:6379"

[database]
url = "postgresql://typf:password@localhost/typf"
max_connections = 20

[security]
api_token_required = true
rate_limit = 100 # requests per minute
max_text_length = 10000

[fonts]
default_font = "Roboto-Regular.ttf"
font_directory = "/app/fonts"
preload_fonts = ["Roboto-Regular.ttf", "OpenSans-Regular.ttf"]
```

### Dynamic Configuration

```rust
use config::{Config, File, Environment};

struct AppConfig {
    server: ServerConfig,
    cache: CacheConfig,
    database: DatabaseConfig,
    security: SecurityConfig,
    fonts: FontConfig,
}

impl AppConfig {
    fn from_env() -> Result<Self> {
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/production").required(false))
            .add_source(Environment::with_prefix("TYPF"))
            .build()?;
        
        config.try_deserialize()
    }
    
    fn reload(&mut self) -> Result<()> {
        let new_config = Self::from_env()?;
        *self = new_config;
        Ok(())
    }
}
```

---

Deploy TypF as a scalable web service with proper monitoring, caching, and security. Use container orchestration for production scaling and implement comprehensive observability to maintain performance.
