# Chapter 23: Deployment and Integration

## Overview

Deploying TYPF effectively requires understanding various deployment patterns, integration strategies, and operational considerations. This chapter covers comprehensive deployment strategies for different environments, from embedded systems to cloud services, along with integration patterns for web applications, mobile apps, and enterprise systems. Whether you're building a simple proof-of-concept or a production-scale text processing service, this chapter provides the deployment knowledge needed for successful TYPF integration.

## Deployment Architectures

### Standalone Binary Deployment

```rust
// Single binary deployment with embedded resources
#[cfg(feature = "standalone")]
pub struct StandaloneDeployment {
    embedded_fonts: HashMap<String, &'static [u8]>,
    configuration: DeploymentConfig,
    runtime: DeploymentRuntime,
}

#[derive(Debug, Clone)]
pub struct DeploymentConfig {
    pub font_directory: Option<PathBuf>,
    pub cache_directory: Option<PathBuf>,
    pub max_memory_mb: usize,
    pub worker_threads: usize,
    pub enable_telemetry: bool,
    pub log_level: LogLevel,
}

impl StandaloneDeployment {
    pub fn new(config: DeploymentConfig) -> Result<Self, TypfError> {
        let embedded_fonts = Self::load_embedded_fonts()?;
        let runtime = DeploymentRuntime::initialize(&config)?;
        
        Ok(Self {
            embedded_fonts,
            configuration: config,
            runtime,
        })
    }
    
    pub fn start_server(self, bind_address: SocketAddr) -> Result<ServerHandle, TypfError> {
        let app = self.create_web_application()?;
        let server = axum::Server::bind(&bind_address)
            .serve(app.into_make_service());
            
        Ok(ServerHandle::new(server))
    }
    
    fn load_embedded_fonts() -> Result<HashMap<String, &'static [u8]>, TypfError> {
        let mut fonts = HashMap::new();
        
        // Embed fonts using include_bytes! at compile time
        fonts.insert("inter-regular".to_string(), include_bytes!("../fonts/Inter-Regular.ttf"));
        fonts.insert("inter-bold".to_string(), include_bytes!("../fonts/Inter-Bold.ttf"));
        fonts.insert("noto-arabic".to_string(), include_bytes!("../fonts/NotoSansArabic-Regular.ttf"));
        fonts.insert("noto-japanese".to_string(), include_bytes!("../fonts/NotoSansJP-Regular.ttf"));
        
        Ok(fonts)
    }
    
    pub fn get_font_bytes(&self, font_name: &str) -> Option<&'static [u8]> {
        self.embedded_fonts.get(font_name).copied()
    }
    
    fn create_web_application(&self) -> Result<axum::Router, TypfError> {
        let app = axum::Router::new()
            .route("/render", axum::post(render_text_handler))
            .route("/fonts", axum::get(list_fonts_handler))
            .route("/health", axum::get(health_check_handler))
            .layer(axum::extract::Extension(Arc::new(self.clone())))
            .layer(tower_http::trace::TraceLayer::new_for_http());
            
        Ok(app)
    }
}

// Web handlers
async fn render_text_handler(
    axum::extract::Extension(deployment): axum::extract::Extension<Arc<StandaloneDeployment>>,
    axum::Json(request): axum::Json<RenderRequest>,
) -> Result<axum::Json<RenderResponse>, TypfError> {
    let pipeline = PipelineBuilder::new()
        .with_shaper(&request.shaper)
        .with_renderer(&request.renderer)
        .build()?;
    
    let result = pipeline.render_text(
        &request.text,
        &request.font,
        request.font_size,
    )?;
    
    let base64_image = base64::encode(&result.data);
    
    Ok(axum::Json(RenderResponse {
        image: base64_image,
        width: result.width,
        height: result.height,
        format: result.format,
    }))
}

#[derive(Deserialize)]
pub struct RenderRequest {
    pub text: String,
    pub font: String,
    pub font_size: f32,
    pub shaper: String,
    pub renderer: String,
}

#[derive(Serialize)]
pub struct RenderResponse {
    pub image: String, // Base64 encoded
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}
```

### Container Deployment

```dockerfile
# Multi-stage Dockerfile for optimal image size
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libfontconfig1-dev \
    libharfbuzz-dev \
    libskia-dev \
    && rm -rf /var/lib/apt/lists/*

# Build with minimal features
RUN cargo build --release --features="shaping-hb,render-skia,export-png" --no-default-features

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libfontconfig1 \
    libharfbuzz0b \
    libskia0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false typf
WORKDIR /app

# Copy binary and set permissions
COPY --from=builder /app/target/release/typf-server /usr/local/bin/typf-server
COPY --from=builder /app/fonts /app/fonts

# Set up configuration
COPY config/production.toml /app/config.toml

# Create cache directory
RUN mkdir -p /app/cache && chown -R typf:typf /app

USER typf

EXPOSE 8080

CMD ["typf-server", "--config", "/app/config.toml", "--bind", "0.0.0.0:8080"]
```

```yaml
# Kubernetes deployment configuration
apiVersion: apps/v1
kind: Deployment
metadata:
  name: typf-server
  labels:
    app: typf
    version: v2.0.0
spec:
  replicas: 3
  selector:
    matchLabels:
      app: typf
  template:
    metadata:
      labels:
        app: typf
        version: v2.0.0
    spec:
      containers:
      - name: typf-server
        image: typf/typf-server:2.0.0
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: RUST_LOG
          value: "info,typf=debug"
        - name: TYPF_CACHE_DIR
          value: "/cache"
        - name: TYPF_MAX_MEMORY_MB
          value: "512"
        - name: TYPF_WORKER_THREADS
          value: "4"
        resources:
          requests:
            memory: "256Mi"
            cpu: "200m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        volumeMounts:
        - name: cache-volume
          mountPath: /cache
        - name: fonts-volume
          mountPath: /fonts
          readOnly: true
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: cache-volume
        emptyDir:
          sizeLimit: 100Mi
      - name: fonts-volume
        configMap:
          name: typf-fonts

---
apiVersion: v1
kind: Service
metadata:
  name: typf-service
spec:
  selector:
    app: typf
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: ClusterIP

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: typf-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: typf-server
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Cloud Function Deployment

```rust
// Cloud function deployment for serverless platforms
#[cfg(feature = "cloud-function")]
pub mod cloud_function {
    use super::*;
    use lambda_runtime::{Error as LambdaError, LambdaEvent};
    use serde_json::{json, Value};
    
    pub async fn render_text_function(
        event: LambdaEvent<Value>,
    ) -> Result<Value, LambdaError> {
        let request: RenderRequest = serde_json::from_value(event.payload)?;
        
        // Initialize pipeline once
        let pipeline = get_or_create_pipeline(&request)?;
        
        // Render text
        let result = pipeline.render_text(
            &request.text,
            &request.font,
            request.font_size,
        )?;
        
        // Convert to response format
        let response = json!({
            "statusCode": 200,
            "body": base64::encode(&result.data),
            "headers": {
                "Content-Type": "image/png",
                "X-Image-Width": result.width,
                "X-Image-Height": result.height,
            },
            "isBase64Encoded": true
        });
        
        Ok(response)
    }
    
    static PIPELINE_CACHE: std::sync::OnceLock<std::collections::HashMap<String, Arc<Pipeline>>> = 
        std::sync::OnceLock::new();
    
    fn get_or_create_pipeline(request: &RenderRequest) -> Result<Arc<Pipeline>, LambdaError> {
        let cache = PIPELINE_CACHE.get_or_init(|| std::collections::HashMap::new());
        let cache_key = format!("{}-{}", request.shaper, request.renderer);
        
        if let Some(pipeline) = cache.get(&cache_key) {
            return Ok(Arc::clone(pipeline));
        }
        
        // Create new pipeline
        let pipeline = PipelineBuilder::new()
            .with_shaper(&request.shaper)
            .with_renderer(&request.renderer)
            .build()
            .map_err(|e| LambdaError::from(e.to_string()))?;
        
        let pipeline_arc = Arc::new(pipeline);
        cache.insert(cache_key, pipeline_arc.clone());
        
        Ok(pipeline_arc)
    }
}

// AWS Lambda specific configuration
#[cfg(feature = "aws-lambda")]
pub mod aws_lambda_config {
    use lambda_runtime::{run, service_fn};
    
    #[tokio::main]
    async fn main() -> Result<(), lambda_runtime::Error> {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .init();
        
        run(service_fn(cloud_function::render_text_function)).await
    }
}

// Google Cloud Functions configuration
#[cfg(feature = "gcloud-functions")]
#[no_mangle]
pub extern "C" fn render_text_gcf(ptr: *mut u8, len: usize) -> *mut u8 {
    // Implementation for Google Cloud Functions
    // This would handle the specific memory management requirements
    ptr
}
```

## Integration Patterns

### Web Framework Integration

```rust
// Axum integration example
pub struct TypfAxumIntegration {
    pipeline_cache: Arc<RwLock<HashMap<String, Arc<Pipeline>>>>,
    font_loader: Arc<FontLoader>,
}

impl TypfAxumIntegration {
    pub fn new() -> Self {
        Self {
            pipeline_cache: Arc::new(RwLock::new(HashMap::new())),
            font_loader: Arc::new(FontLoader::new()),
        }
    }
    
    pub fn create_router(self) -> axum::Router {
        axum::Router::new()
            .route("/api/v1/render", axum::post(render_handler))
            .route("/api/v1/render/batch", axum::post(render_batch_handler))
            .route("/api/v1/fonts", axum::get(list_fonts_handler))
            .route("/api/v1/metrics", axum::get(metrics_handler))
            .layer(axum::extract::Extension(Arc::new(self)))
            .layer(tower_http::cors::CorsLayer::permissive())
            .layer(tower_http::trace::TraceLayer::new_for_http())
    }
}

async fn render_handler(
    axum::Extension(integration): axum::Extension<Arc<TypfAxumIntegration>>,
    axum::Json(request): axum::Json<RenderRequest>,
) -> Result<axum::response::Response, TypfError> {
    let pipeline = integration.get_pipeline(&request.shaper, &request.renderer)?;
    
    let result = pipeline.render_text(
        &request.text,
        &request.font,
        request.font_size,
    )?;
    
    let content_type = match result.format {
        PixelFormat::Png => "image/png",
        PixelFormat::Jpeg => "image/jpeg",
        PixelFormat::Svg => "image/svg+xml",
        _ => "application/octet-stream",
    };
    
    Ok(axum::response::Response::builder()
        .status(200)
        .header("Content-Type", content_type)
        .header("X-Image-Width", result.width.to_string())
        .header("X-Image-Height", result.height.to_string())
        .body(axum::body::Body::from(result.data))?)
}

// Actix-web integration
#[cfg(feature = "actix-web")]
pub mod actix_integration {
    use actix_web::{web, App, HttpResponse, HttpServer, Responder};
    use super::*;
    
    pub struct TypfActixIntegration {
        pipeline_cache: Arc<RwLock<HashMap<String, Arc<Pipeline>>>>,
    }
    
    impl TypfActixIntegration {
        pub fn new() -> Self {
            Self {
                pipeline_cache: Arc::new(RwLock::new(HashMap::new())),
            }
        }
        
        pub async fn render_text(
            integration: web::Data<Arc<TypfActixIntegration>>,
            request: web::Json<RenderRequest>,
        ) -> impl Responder {
            match integration.get_pipeline(&request.shaper, &request.renderer) {
                Ok(pipeline) => {
                    match pipeline.render_text(&request.text, &request.font, request.font_size) {
                        Ok(result) => {
                            HttpResponse::Ok()
                                .content_type("image/png")
                                .header("X-Image-Width", result.width.to_string())
                                .header("X-Image-Height", result.height.to_string())
                                .body(result.data)
                        }
                        Err(e) => HttpResponse::InternalServerError().json(json!({
                            "error": e.to_string()
                        })),
                    }
                }
                Err(e) => HttpResponse::BadRequest().json(json!({
                    "error": e.to_string()
                })),
            }
        }
        
        pub fn create_server(self, bind_address: &str) -> Result<actix_web::dev::Server, TypfError> {
            let app_data = web::Data::new(Arc::new(self));
            
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(app_data.clone())
                    .route("/api/v1/render", web::post().to(Self::render_text))
                    .route("/health", web::get().to(HttpResponse::Ok))
            })
            .bind(bind_address)?
            .run();
            
            Ok(server)
        }
    }
}
```

### Database Integration

```rust
// Database-backed font management
pub struct DatabaseFontManager {
    pool: sqlx::PgPool,
    cache: Arc<RwLock<HashMap<String, Arc<FontData>>>>,
}

impl DatabaseFontManager {
    pub async fn new(database_url: &str) -> Result<Self, TypfError> {
        let pool = sqlx::PgPool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    pub async fn store_font(&self, 
        name: &str, 
        family: &str, 
        style: &str,
        data: &[u8],
        metadata: &FontMetadata,
    ) -> Result<uuid::Uuid, TypfError> {
        let id = uuid::Uuid::new_v4();
        
        sqlx::query!(
            r#"
            INSERT INTO fonts (id, name, family, style, data, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#,
            id,
            name,
            family,
            style,
            data,
            serde_json::to_value(metadata)?,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(id)
    }
    
    pub async fn load_font(&self, id: uuid::Uuid) -> Result<FontData, TypfError> {
        // Check cache first
        let cache_key = id.to_string();
        if let Ok(cache) = self.cache.read() {
            if let Some(font_data) = cache.get(&cache_key) {
                return Ok((*font_data).clone());
            }
        }
        
        // Load from database
        let row = sqlx::query!(
            "SELECT data, metadata FROM fonts WHERE id = $1",
            id
        )
        .fetch_one(&self.pool)
        .await?;
        
        let font_data = FontData::from_bytes(row.data)?;
        
        // Cache the result
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(cache_key, Arc::new(font_data.clone()));
        }
        
        Ok(font_data)
    }
    
    pub async fn search_fonts(&self, 
        family: Option<&str>, 
        style: Option<&str>,
        script: Option<&str>,
    ) -> Result<Vec<FontInfo>, TypfError> {
        let mut query = "SELECT id, name, family, style, metadata FROM fonts WHERE 1=1".to_string();
        let mut params = Vec::new();
        let mut param_index = 1;
        
        if let Some(family) = family {
            query.push_str(&format!(" AND family ILIKE ${}", param_index));
            params.push(format!("%{}%", family));
            param_index += 1;
        }
        
        if let Some(style) = style {
            query.push_str(&format!(" AND style ILIKE ${}", param_index));
            params.push(format!("%{}%", style));
            param_index += 1;
        }
        
        // Execute query with dynamic parameters
        let mut query_builder = sqlx::query_builder::QueryBuilder::new(&query);
        
        for param in &params {
            query_builder.push_bind(param);
        }
        
        let rows = query_builder.build().fetch_all(&self.pool).await?;
        
        let mut fonts = Vec::new();
        for row in rows {
            let metadata: FontMetadata = serde_json::from_value(row.metadata)?;
            
            // Filter by script metadata
            if let Some(script) = script {
                if !metadata.supported_scripts.contains(&script.to_string()) {
                    continue;
                }
            }
            
            fonts.push(FontInfo {
                id: row.id,
                name: row.name,
                family: row.family,
                style: row.style,
                metadata,
            });
        }
        
        Ok(fonts)
    }
}

// Redis-based caching for shaping results
pub struct RedisShapeCache {
    client: redis::Client,
    ttl_seconds: u64,
}

impl RedisShapeCache {
    pub fn new(redis_url: &str, ttl_seconds: u64) -> Result<Self, TypfError> {
        let client = redis::Client::open(redis_url)?;
        
        Ok(Self {
            client,
            ttl_seconds,
        })
    }
    
    async fn generate_cache_key(&self, 
        text: &str, 
        font_id: &str, 
        font_size: f32,
        options: &ShapingOptions,
    ) -> Result<String, TypfError> {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        font_id.hash(&mut hasher);
        (font_size.to_bits()).hash(&mut hasher);
        serde_json::to_string(options)?.hash(&mut hasher);
        
        Ok(format!("shape:{:x}", hasher.finish()))
    }
    
    pub async fn get(&self, key: &str) -> Result<Option<ShapingResult>, TypfError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let cached_data: Option<Vec<u8>> = redis::Cmd::get(key).query_async(&mut conn).await?;
        
        match cached_data {
            Some(data) => {
                let result: ShapingResult = bincode::deserialize(&data)?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }
    
    pub async fn set(&self, 
        key: &str, 
        result: &ShapingResult,
    ) -> Result<(), TypfError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let serialized = bincode::serialize(result)?;
        
        redis::Cmd::set_ex(key, serialized, self.ttl_seconds)
            .query_async(&mut conn)
            .await?;
        
        Ok(())
    }
}
```

### Message Queue Integration

```rust
// RabbitMQ integration for batch processing
pub struct BatchProcessor {
    connection: lapin::Connection,
    channel: lapin::Channel,
    pipeline_factory: Arc<PipelineFactory>,
}

impl BatchProcessor {
    pub async fn new(amqp_url: &str) -> Result<Self, TypfError> {
        let connection = lapin::Connection::connect(amqp_url, lapin::ConnectionProperties::default()).await?;
        let channel = connection.create_channel().await?;
        
        // Declare queues
        channel.queue_declare(
            "render_requests",
            lapin::options::QueueDeclareOptions::default(),
            lapin::types::FieldTable::default(),
        ).await?;
        
        channel.queue_declare(
            "render_results",
            lapin::options::QueueDeclareOptions::default(),
            lapin::types::FieldTable::default(),
        ).await?;
        
        Ok(Self {
            connection,
            channel,
            pipeline_factory: Arc::new(PipelineFactory::new()),
        })
    }
    
    pub async fn start_consuming(&mut self) -> Result<(), TypfError> {
        let consumer = self.channel
            .basic_consume(
                "render_requests",
                "typf_processor",
                lapin::options::BasicConsumeOptions::default(),
                lapin::types::FieldTable::default(),
            )
            .await?;
        
        consumer.set_delegate(move |delivery: lapin::message::Delivery| async move {
            if let Err(e) = self.process_message(delivery).await {
                log::error!("Failed to process message: {}", e);
            }
        });
        
        Ok(())
    }
    
    async fn process_message(&self, delivery: lapin::message::Delivery) -> Result<(), TypfError> {
        let request: BatchRenderRequest = serde_json::from_slice(&delivery.data)?;
        
        // Process batch
        let results = self.process_batch(&request).await?;
        
        // Send results back
        let response = BatchRenderResponse {
            batch_id: request.batch_id,
            results,
            completed_at: chrono::Utc::now(),
        };
        
        let response_data = serde_json::to_vec(&response)?;
        
        self.channel.basic_publish(
            "",
            "render_results",
            lapin::options::BasicPublishOptions::default(),
            response_data,
            lapin::types::BasicProperties::default(),
        ).await?;
        
        // Acknowledge original message
        self.channel.basic_ack(delivery.delivery_tag, lapin::options::BasicAckOptions::default()).await?;
        
        Ok(())
    }
    
    async fn process_batch(&self, request: &BatchRenderRequest) -> Result<Vec<RenderResult>, TypfError> {
        let pipeline = self.pipeline_factory.create_pipeline(
            &request.shaper,
            &request.renderer,
        )?;
        
        let mut results = Vec::with_capacity(request.items.len());
        
        for item in &request.items {
            let render_result = match pipeline.render_text(
                &item.text,
                &item.font,
                item.font_size,
            ) {
                Ok(output) => RenderResult {
                    item_id: item.id.clone(),
                    success: true,
                    data: Some(output),
                    error: None,
                },
                Err(e) => RenderResult {
                    item_id: item.id.clone(),
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                },
            };
            
            results.push(render_result);
        }
        
        Ok(results)
    }
}

#[derive(Deserialize)]
pub struct BatchRenderRequest {
    pub batch_id: String,
    pub shaper: String,
    pub renderer: String,
    pub items: Vec<RenderItem>,
}

#[derive(Deserialize)]
pub struct RenderItem {
    pub id: String,
    pub text: String,
    pub font: String,
    pub font_size: f32,
}

#[derive(Serialize)]
pub struct BatchRenderResponse {
    pub batch_id: String,
    pub results: Vec<RenderResult>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct RenderResult {
    pub item_id: String,
    pub success: bool,
    pub data: Option<RenderOutput>,
    pub error: Option<String>,
}
```

## Monitoring and Observability

### Metrics Collection

```rust
// Prometheus metrics integration
pub struct MetricsCollector {
    registry: prometheus::Registry,
    render_counter: prometheus::IntCounterVec,
    render_duration: prometheus::HistogramVec,
    cache_hit_counter: prometheus::IntCounterVec,
    memory_usage: prometheus::GaugeVec,
}

impl MetricsCollector {
    pub fn new() -> Result<Self, TypfError> {
        let registry = prometheus::Registry::new();
        
        let render_counter = prometheus::IntCounterVec::new(
            prometheus::Opts::new("typf_renders_total", "Total number of renders"),
            &["shaper", "renderer", "status"],
        )?;
        
        let render_duration = prometheus::HistogramVec::new(
            prometheus::HistogramOpts::new("typf_render_duration_seconds", "Render duration in seconds"),
            &["shaper", "renderer"],
        )?;
        
        let cache_hit_counter = prometheus::IntCounterVec::new(
            prometheus::Opts::new("typf_cache_hits_total", "Total cache hits"),
            &["cache_type"],
        )?;
        
        let memory_usage = prometheus::GaugeVec::new(
            prometheus::Opts::new("typf_memory_usage_bytes", "Memory usage in bytes"),
            &["component"],
        )?;
        
        registry.register(Box::new(render_counter.clone()))?;
        registry.register(Box::new(render_duration.clone()))?;
        registry.register(Box::new(cache_hit_counter.clone()))?;
        registry.register(Box::new(memory_usage.clone()))?;
        
        Ok(Self {
            registry,
            render_counter,
            render_duration,
            cache_hit_counter,
            memory_usage,
        })
    }
    
    pub fn record_render(&self, 
        shaper: &str, 
        renderer: &str, 
        success: bool,
        duration: Duration,
    ) {
        let status = if success { "success" } else { "error" };
        self.render_counter.with_label_values(&[shaper, renderer, status]).inc();
        self.render_duration.with_label_values(&[shaper, renderer]).observe(duration.as_secs_f64());
    }
    
    pub fn record_cache_hit(&self, cache_type: &str) {
        self.cache_hit_counter.with_label_values(&[cache_type]).inc();
    }
    
    pub fn update_memory_usage(&self, component: &str, bytes: u64) {
        self.memory_usage.with_label_values(&[component]).set(bytes as f64);
    }
    
    pub fn gather(&self) -> Result<String, TypfError> {
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let buffer = encoder.encode_to_string(&metric_families)?;
        
        Ok(buffer)
    }
}

// Health check system
pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck + Send + Sync>>,
}

pub trait HealthCheck {
    fn check(&self) -> Result<HealthStatus, TypfError>;
    fn name(&self) -> &str;
}

#[derive(Debug)]
pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metrics: std::collections::HashMap<String, serde_json::Value>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }
    
    pub fn add_check(&mut self, check: Box<dyn HealthCheck + Send + Sync>) {
        self.checks.push(check);
    }
    
    pub async fn run_all_checks(&self) -> Result<OverallHealth, TypfError> {
        let mut check_results = Vec::new();
        let mut overall_healthy = true;
        
        for check in &self.checks {
            match check.check() {
                Ok(status) => {
                    if !status.healthy {
                        overall_healthy = false;
                    }
                    check_results.push(status);
                }
                Err(e) => {
                    overall_healthy = false;
                    check_results.push(HealthStatus {
                        healthy: false,
                        message: e.to_string(),
                        timestamp: chrono::Utc::now(),
                        metrics: std::collections::HashMap::new(),
                    });
                }
            }
        }
        
        Ok(OverallHealth {
            healthy: overall_healthy,
            timestamp: chrono::Utc::now(),
            checks: check_results,
        })
    }
}

#[derive(Serialize)]
pub struct OverallHealth {
    pub healthy: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub checks: Vec<HealthStatus>,
}

// Database health check
pub struct DatabaseHealthCheck {
    pool: sqlx::PgPool,
}

impl HealthCheck for DatabaseHealthCheck {
    fn check(&self) -> Result<HealthStatus, TypfError> {
        let start_time = std::time::Instant::now();
        
        let result: Result<Option<i64>, sqlx::Error> = sqlx::query_scalar("SELECT 1")
            .fetch_one(&self.pool)
            .await;
        
        let duration = start_time.elapsed();
        
        match result {
            Ok(Some(_)) => {
                let mut metrics = std::collections::HashMap::new();
                metrics.insert("query_duration_ms".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(duration.as_millis())
                ));
                
                Ok(HealthStatus {
                    healthy: true,
                    message: "Database connection successful".to_string(),
                    timestamp: chrono::Utc::now(),
                    metrics,
                })
            }
            Err(e) => Ok(HealthStatus {
                healthy: false,
                message: format!("Database connection failed: {}", e),
                timestamp: chrono::Utc::now(),
                metrics: std::collections::HashMap::new(),
            }),
        }
    }
    
    fn name(&self) -> &str {
        "database"
    }
}
```

## Configuration Management

### Environment-based Configuration

```rust
// Configuration management with multiple sources
pub struct ConfigurationManager {
    config: Config,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub cache: CacheConfig,
    pub fonts: FontConfig,
    pub metrics: MetricsConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub worker_threads: usize,
    pub max_connections: usize,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    pub font_cache_size_mb: usize,
    pub shape_cache_size_mb: usize,
    pub enable_compression: bool,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FontConfig {
    pub font_directory: String,
    pub auto_discover: bool,
    pub supported_formats: Vec<String>,
}

impl ConfigurationManager {
    pub fn load() -> Result<Self, TypfError> {
        let config = config::Config::builder()
            // Load default configuration
            .add_source(config::File::with_name("config/default"))
            // Override with environment-specific config
            .add_source(config::File::with_name(&format!("config/{}", 
                std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
            )).required(false))
            // Override with environment variables
            .add_source(config::Environment::with_prefix("TYPF"))
            .build()?;
        
        let config: Config = config.try_deserialize()?;
        
        // Validate configuration
        Self::validate_config(&config)?;
        
        Ok(Self { config })
    }
    
    fn validate_config(config: &Config) -> Result<(), TypfError> {
        if config.server.worker_threads == 0 {
            return Err(TypfError::InvalidConfiguration(
                "server.worker_threads must be greater than 0".to_string()
            ));
        }
        
        if !std::path::Path::new(&config.fonts.font_directory).exists() {
            return Err(TypfError::InvalidConfiguration(
                format!("Font directory does not exist: {}", config.fonts.font_directory)
            ));
        }
        
        Ok(())
    }
    
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

// Configuration hot reloading
pub struct ConfigWatcher {
    config_path: PathBuf,
    current_config: Arc<RwLock<Config>>,
    watch_sender: tokio::sync::mpsc::UnboundedSender<Config>,
}

impl ConfigWatcher {
    pub fn new(config_path: PathBuf) -> Result<Self, TypfError> {
        let initial_config = Self::load_config(&config_path)?;
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        
        let watcher = Self {
            config_path,
            current_config: Arc::new(RwLock::new(initial_config)),
            watch_sender: sender,
        };
        
        // Start watching for changes
        watcher.start_watching()?;
        
        Ok(watcher)
    }
    
    fn load_config(path: &PathBuf) -> Result<Config, TypfError> {
        let config_content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }
    
    fn start_watching(&self) -> Result<(), TypfError> {
        use notify::{Watcher, RecursiveMode, Event, EventKind};
        
        let config_path = self.config_path.clone();
        let sender = self.watch_sender.clone();
        
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if event.kind == EventKind::Modify {
                        if let Ok(new_config) = Self::load_config(&config_path) {
                            let _ = sender.send(new_config);
                        }
                    }
                }
                Err(e) => {
                    log::error!("Config watch error: {}", e);
                }
            }
        })?;
        
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        
        Ok(())
    }
    
    pub async fn get_config_updates(&self) -> impl Stream<Item = Config> {
        let mut receiver = self.watch_sender.subscribe();
        async_stream::stream! {
            while let Some(config) = receiver.recv().await {
                yield config;
            }
        }
    }
}
```

## Security Considerations

### Authentication and Authorization

```rust
// JWT-based authentication for API endpoints
pub struct AuthMiddleware {
    jwt_secret: String,
}

impl AuthMiddleware {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }
    
    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let validation = Validation::default();
        
        decode::<Claims>(token, &key, &validation)
            .map(|data| data.claims)
            .map_err(|_| AuthError::InvalidToken)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    User,
    ReadOnly,
}

// Rate limiting
pub struct RateLimiter {
    limiter: Arc<governor::DefaultDirectRateLimiter>,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        let limiter = Arc::new(
            governor::RateLimiter::direct(
                governor::Quota::per_second(std::num::NonZeroU32::new(requests_per_second).unwrap())
            )
        );
        
        Self { limiter }
    }
    
    pub async fn check_rate_limit(&self, client_ip: &str) -> Result<(), RateLimitError> {
        match self.limiter.check() {
            Ok(_) => Ok(()),
            Err(negative) => Err(RateLimitError::RateLimited {
                retry_after: negative.wait_time_from(Instant::now()).as_secs(),
            }),
        }
    }
}

// Input validation and sanitization
pub struct InputValidator {
    max_text_length: usize,
    allowed_extensions: Vec<String>,
}

impl InputValidator {
    pub fn new(max_text_length: usize) -> Self {
        Self {
            max_text_length,
            allowed_extensions: vec![
                "ttf".to_string(),
                "otf".to_string(), 
                "woff".to_string(),
                "woff2".to_string(),
            ],
        }
    }
    
    pub fn validate_render_request(&self, request: &RenderRequest) -> Result<(), ValidationError> {
        // Validate text length
        if request.text.len() > self.max_text_length {
            return Err(ValidationError::TextTooLong {
                actual: request.text.len(),
                max: self.max_text_length,
            });
        }
        
        // Validate font size
        if request.font_size <= 0.0 || request.font_size > 1000.0 {
            return Err(ValidationError::InvalidFontSize(request.font_size));
        }
        
        // Validate font path to prevent directory traversal
        if request.font.contains("..") || request.font.starts_with('/') {
            return Err(ValidationError::UnsafeFontPath(request.font.clone()));
        }
        
        // Validate shaper/renderer names
        let valid_shapers = vec!["harfbuzz", "icu-hb", "none", "coretext", "directwrite"];
        let valid_renderers = vec!["skia", "orge", "coregraphics", "direct2d", "zeno"];
        
        if !valid_shapers.contains(&request.shaper.as_str()) {
            return Err(ValidationError::InvalidShaper(request.shaper.clone()));
        }
        
        if !valid_renderers.contains(&request.renderer.as_str()) {
            return Err(ValidationError::InvalidRenderer(request.renderer.clone()));
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Text too long: {actual} characters (max: {max})")]
    TextTooLong { actual: usize, max: usize },
    
    #[error("Invalid font size: {0}")]
    InvalidFontSize(f32),
    
    #[error("Unsafe font path: {0}")]
    UnsafeFontPath(String),
    
    #[error("Invalid shaper: {0}")]
    InvalidShaper(String),
    
    #[error("Invalid renderer: {0}")]
    InvalidRenderer(String),
}
```

By implementing these comprehensive deployment and integration patterns, TYPF can be successfully deployed in any environment from embedded systems to cloud platforms, with proper monitoring, security, and operational considerations. The modular architecture ensures that deployment can be tailored to specific requirements while maintaining core functionality and performance characteristics.