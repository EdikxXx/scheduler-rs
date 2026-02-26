# scheduler-rs

A distributed task scheduler built with Rust using Event-Driven Architecture (EDA).

## Overview

This project implements a robust, asynchronous task scheduler with the following characteristics:

- **Event-Driven Architecture (EDA)**: Decoupled components communicate via events
- **Type-State Pattern**: Task lifecycle enforced at compile time
- **Async/Await**: Built on Tokio for high-performance async execution
- **Distributed-Ready**: Pluggable backends for broker and storage
- **Fault-Tolerant**: Retry mechanisms, circuit breakers, and dead letter queues

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Scheduler Core                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   Scheduler  │  │    Broker    │  │   Storage    │       │
│  │   (Runtime)  │──│  (Message    │──│  (Persistence│       │
│  │              │  │   Queue)     │  │              │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
         │                     │                     │
         ▼                     ▼                     ▼
   ┌──────────┐        ┌──────────┐         ┌──────────┐
   │  Events  │        │  Topics  │         │  Tasks   │
   │Scheduled,│        │          │         │  State   │
   │Started,  │        │          │         │ Machine  │
   │Completed,│        │          │         │          │
   │Failed    │        │          │         │          │
   └──────────┘        └──────────┘         └──────────┘
```

### Components

1. **Scheduler**: Main orchestrator with event loop and worker pool
2. **Broker**: Message queue for event distribution (In-Memory, Redis, SQS)
3. **Storage**: Persistence layer for task state (In-Memory, JSON, PostgreSQL)
4. **Tasks**: Type-state pattern for lifecycle management
5. **Events**: Immutable records of state transitions

## Development Roadmap

### Phase 1: Core Infrastructure (Weeks 1-2)

Build the foundational components that everything else depends on.

#### 1.1 Complete Broker Implementation

**Files**: `src/brokers/utils.rs`, `src/brokers/types.rs`

**Step 1.1.1: Implement `consume()` method**
- Create async stream for message consumption
- Return `impl Stream<Item = Message<T>>`
- Support multiple concurrent consumers
- Add consumer identification

**Step 1.1.2: Implement `ack()` method**
- Add message acknowledgment tracking
- Implement at-least-once delivery semantics
- Track unacknowledged messages with timeouts
- Add automatic redelivery for unacked messages

**Step 1.1.3: Add Dead Letter Queue (DLQ)**
- Automatically create `dlq` topic
- Route messages after max retry attempts
- Add DLQ inspection and replay methods
- Implement DLQ metrics and alerting hooks

**Deliverables**:
- [ ] `consume()` returns working message stream
- [ ] `ack()` properly tracks message delivery
- [ ] Failed messages routed to DLQ after retries
- [ ] Unit tests for all broker operations

#### 1.2 Implement Storage Backends

**Files**: `src/storages/memory.rs` (new), `src/storages/types.rs`

**Step 1.2.1: Implement InMemoryStorage**
```rust
pub struct InMemoryStorage {
    tasks: DashMap<Uuid, TaskRecord>,
    status_index: DashMap<TaskStatus, Vec<Uuid>>,
    scheduled_index: BTreeMap<OffsetDateTime, Vec<Uuid>>,
}
```
- Store tasks with full state
- Maintain indexes for efficient queries
- Thread-safe concurrent access

**Step 1.2.2: Add persistence layer**
- Serialize to JSON on state changes
- Auto-save with configurable intervals
- Load state on startup
- Handle corruption gracefully

**Step 1.2.3: Implement query methods**
- `find_by_status(status)` - Get all tasks in a state
- `find_due(before)` - Get tasks scheduled before time
- `find_by_id(id)` - Get specific task
- `find_overdue()` - Get tasks past scheduled time

**Deliverables**:
- [ ] `InMemoryStorage` fully implements `Storage` trait
- [ ] JSON persistence working
- [ ] All query methods functional
- [ ] Storage survives process restart

#### 1.3 Complete Task State Machine

**Files**: `src/tasks/utils.rs`, `src/tasks/types.rs`

**Step 1.3.1: Implement all state transitions**

| Method | From State | To State | Description |
|--------|-----------|----------|-------------|
| `dispatch()` | Waiting | Running | Assign to worker |
| `complete()` | Running | Success | Mark as finished |
| `fail()` | Running | Failed | Record error |
| `retry()` | Running | Retrying | Queue for retry |
| `reschedule()` | Retrying | Waiting | Back to queue |

**Step 1.3.2: Add retry configuration**
```rust
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_strategy: BackoffStrategy,
    pub initial_delay: Duration,
}

pub enum BackoffStrategy {
    Fixed(Duration),
    Linear { multiplier: f64 },
    Exponential { base: f64, max_delay: Duration },
}
```

**Deliverables**:
- [ ] All state transitions compile and work
- [ ] Retry policy configurable per task
- [ ] State changes emit events
- [ ] Invalid transitions prevented at compile time

---

### Phase 2: Scheduler Runtime (Weeks 3-4)

Build the execution engine that orchestrates tasks.

#### 2.1 Implement Main Scheduler Loop

**Files**: `src/scheduler/runtime.rs` (new)

**Step 2.1.1: Create event loop**
```rust
pub struct SchedulerRuntime<B: Broker, S: Storage> {
    broker: Arc<B>,
    storage: Arc<S>,
    config: SchedulerConfig,
    shutdown: broadcast::Sender<()>,
}

impl<B: Broker, S: Storage> SchedulerRuntime<B, S> {
    pub async fn run(&self) -> anyhow::Result<()> {
        let mut interval = tokio::time::interval(self.config.poll_interval);
        let mut shutdown_rx = self.shutdown.subscribe();
        
        loop {
            tokio::select! {
                _ = interval.tick() => self.process_due_tasks().await?,
                _ = shutdown_rx.recv() => {
                    self.graceful_shutdown().await?;
                    break;
                }
            }
        }
        Ok(())
    }
}
```

**Step 2.1.2: Add graceful shutdown**
- Listen for SIGTERM/SIGINT signals
- Stop accepting new tasks
- Wait for in-progress tasks to complete
- Save final state
- Timeout after configurable duration

**Deliverables**:
- [ ] Event loop running continuously
- [ ] Shutdown signal handled properly
- [ ] No task loss during shutdown
- [ ] Integration tests pass

#### 2.2 Implement Worker Pool

**Files**: `src/scheduler/worker.rs` (new)

**Step 2.2.1: Create worker pool**
```rust
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_queue: mpsc::Receiver<Task>,
    semaphore: Arc<Semaphore>,
}

pub struct Worker {
    id: Uuid,
    handle: JoinHandle<()>,
}
```
- Configurable number of workers
- Task queue with backpressure
- Worker health monitoring

**Step 2.2.2: Add task execution**
- Spawn tasks with `tokio::spawn`
- Enforce task timeouts
- Capture results and errors
- Handle panics gracefully

**Step 2.2.3: Add load balancing**
- Round-robin task distribution
- Worker utilization tracking
- Dynamic worker scaling (optional)

**Deliverables**:
- [ ] Worker pool executes tasks concurrently
- [ ] Tasks respect timeout limits
- [ ] Worker failures don't crash scheduler
- [ ] Benchmark showing throughput

#### 2.3 Implement Scheduling Logic

**Files**: `src/scheduler/cron.rs` (new)

**Step 2.3.1: Add cron expression parser**
- Support standard cron: `* * * * * *` (sec min hour day month weekday)
- Parse into `Schedule` struct
- Calculate next execution times
- Handle timezone properly

**Step 2.3.2: Implement task schedulers**
- **One-time**: Execute at specific datetime
- **Recurring**: Cron-based repeating tasks
- **Delayed**: Execute after duration
- **Immediate**: Execute as soon as possible

**Step 2.3.3: Add scheduling API**
```rust
pub trait Scheduler {
    fn schedule_once(&self, task: Task, at: OffsetDateTime) -> Result<Uuid>;
    fn schedule_recurring(&self, task: Task, cron: &str) -> Result<Uuid>;
    fn schedule_delayed(&self, task: Task, delay: Duration) -> Result<Uuid>;
    fn cancel(&self, task_id: Uuid) -> Result<()>;
}
```

**Deliverables**:
- [ ] Cron expressions parse correctly
- [ ] All scheduling types work
- [ ] Tasks trigger at correct times
- [ ] Cancel stops future executions

---

### Phase 3: Event System (Week 5)

Implement comprehensive event handling for observability and extensibility.

#### 3.1 Define Event Types

**Files**: `src/events/mod.rs` (new)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskEvent {
    Scheduled {
        task_id: Uuid,
        task_type: String,
        scheduled_at: OffsetDateTime,
        run_at: OffsetDateTime,
    },
    Started {
        task_id: Uuid,
        worker_id: Uuid,
        started_at: OffsetDateTime,
    },
    Completed {
        task_id: Uuid,
        worker_id: Uuid,
        completed_at: OffsetDateTime,
        duration_ms: u64,
        result: TaskResult,
    },
    Failed {
        task_id: Uuid,
        worker_id: Uuid,
        failed_at: OffsetDateTime,
        error: String,
        retry_count: u32,
        will_retry: bool,
    },
    Cancelled {
        task_id: Uuid,
        cancelled_at: OffsetDateTime,
        reason: String,
    },
    Retrying {
        task_id: Uuid,
        attempt: u32,
        next_attempt_at: OffsetDateTime,
    },
}
```

#### 3.2 Implement Event Bus

**Step 3.2.1: Create event publisher**
- Publish events to typed topics
- Support multiple subscribers per event type
- Async event delivery

**Step 3.2.2: Add event handlers**
```rust
#[async_trait]
pub trait EventHandler<E: Event> {
    async fn handle(&self, event: E) -> Result<()>;
}

// Built-in handlers
pub struct LoggingHandler;
pub struct MetricsHandler;
pub struct WebhookHandler; // optional
```

**Step 3.2.3: Subscribe to events**
```rust
scheduler.on_event::<TaskStarted>(LoggingHandler);
scheduler.on_event::<TaskFailed>(AlertHandler);
```

#### 3.3 Add Event Persistence

**Step 3.3.1: Event store**
- Append-only event log
- Query by task_id, time range, or event type
- Event replay for recovery
- Compaction for old events

**Deliverables**:
- [ ] All state changes emit events
- [ ] Event handlers receive and process events
- [ ] Event store persists all events
- [ ] Events can be replayed

---

### Phase 4: Resilience & Observability (Week 6)

Make the scheduler production-ready with fault tolerance and monitoring.

#### 4.1 Error Handling & Circuit Breakers

**Step 4.1.1: Implement circuit breaker**
```rust
pub struct CircuitBreaker {
    failure_threshold: u32,
    reset_timeout: Duration,
    half_open_max_calls: u32,
    state: CircuitState,
}

enum CircuitState {
    Closed,     // Normal operation
    Open,       // Failing, reject calls
    HalfOpen,   // Testing if recovered
}
```
- Track failure rates per task type
- Open circuit after threshold
- Half-open state for gradual recovery
- Automatic reset on success

**Step 4.1.2: Add jitter to backoff**
- Prevent thundering herd problem
- Randomize retry delays: `delay * (1 + random())`
- Configurable jitter factor

**Step 4.1.3: Global error handling**
- Catch panics in task execution
- Log errors with context
- Notify on critical failures

**Deliverables**:
- [ ] Circuit breaker opens on failures
- [ ] Backoff with jitter prevents spikes
- [ ] Panics caught and logged
- [ ] System degrades gracefully under load

#### 4.2 Observability

**Step 4.2.1: Structured logging with `tracing`**
```rust
#[instrument(skip(self, task), fields(task_id = %task.id, task_type = %task.task_type))]
async fn execute_task(&self, task: Task) -> Result<TaskResult> {
    debug!("Starting task execution");
    // ... execution
    info!(duration_ms = elapsed, "Task completed");
    Ok(result)
}
```
- Span context across async boundaries
- Correlation IDs for request tracing
- Configurable log levels
- JSON output for production

**Step 4.2.2: Metrics collection**
```rust
// Counters
tasks_scheduled_total
tasks_completed_total
tasks_failed_total
tasks_retried_total

// Histograms
task_execution_duration_ms
task_queue_wait_time_ms
task_retry_count

// Gauges
active_workers
task_queue_depth
scheduled_tasks_pending
```

**Step 4.2.3: Health checks**
```rust
pub struct HealthStatus {
    pub status: HealthState, // Healthy | Degraded | Unhealthy
    pub components: HashMap<String, ComponentHealth>,
    pub last_updated: OffsetDateTime,
}
```
- `/health` endpoint for liveness
- `/ready` endpoint for readiness
- Dependency health (broker, storage)

**Deliverables**:
- [ ] Structured logging throughout
- [ ] Metrics endpoint (Prometheus format)
- [ ] Health check endpoints
- [ ] Dashboard showing key metrics

---

### Phase 5: Production Features (Weeks 7-8)

Add features needed for real-world deployment.

#### 5.1 Configuration Management

**Files**: `config/scheduler.yaml`, `src/config.rs` (new)

```yaml
scheduler:
  name: "production-scheduler"
  workers: 10
  poll_interval_ms: 1000
  shutdown_timeout_secs: 30
  
broker:
  type: in_memory # in_memory | redis | sqs
  redis:
    url: "redis://localhost:6379"
    stream_name: "scheduler-events"
  
storage:
  type: json_file # in_memory | json_file | postgres
  json_file:
    path: "./data/tasks.json"
    auto_save_interval_secs: 60
  
retry:
  max_attempts: 3
  backoff_strategy: exponential
  initial_delay_ms: 1000
  max_delay_ms: 60000
  
logging:
  level: info
  format: json
  
metrics:
  enabled: true
  port: 9090
```

**Deliverables**:
- [ ] Config file loading
- [ ] Environment variable overrides
- [ ] Validation of configuration
- [ ] Hot reload (optional)

#### 5.2 Distributed Features

**Step 5.2.1: Redis broker backend**
- Implement `Broker` trait for Redis Streams
- Consumer groups for load balancing
- Cross-instance event distribution
- Persistence across restarts

**Step 5.2.2: PostgreSQL storage**
- Implement `Storage` trait for PostgreSQL
- ACID transactions for state changes
- Efficient queries with indexes
- Connection pooling

**Step 5.2.3: Distributed locking**
```rust
pub trait DistributedLock {
    async fn acquire(&self, lock_name: &str, ttl: Duration) -> Result<LockGuard>;
}
```
- Prevent duplicate task execution
- Redis Redlock algorithm
- PostgreSQL advisory locks
- Automatic lock renewal

**Step 5.2.4: Leader election**
- Single scheduler instance for coordination
- Automatic failover
- Lease-based leadership

**Deliverables**:
- [ ] Redis broker functional
- [ ] PostgreSQL storage working
- [ ] Distributed locks prevent duplicates
- [ ] Multi-instance deployment possible

#### 5.3 Advanced Task Features

**Step 5.3.1: Task priorities**
```rust
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}
```
- Priority queues for task ordering
- Preemption for critical tasks
- Priority inheritance

**Step 5.3.2: Task dependencies**
```rust
pub struct TaskChain {
    pub tasks: Vec<Uuid>,
    pub execution_mode: ExecutionMode, // Sequential | Parallel
}
```
- DAG-based task workflows
- Dependency resolution
- Parallel branch execution
- Failure propagation

**Step 5.3.3: Task constraints**
- Max runtime limits
- Resource requirements
- Affinity/anti-affinity rules
- Rate limiting per task type

**Deliverables**:
- [ ] Priority queues working
- [ ] Task chains execute in order
- [ ] Constraints enforced
- [ ] Complex workflows supported

---

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/scheduler-rs
cd scheduler-rs

# Build the project
cargo build --release

# Run tests
cargo test
```

### Basic Usage

```rust
use scheduler_rs::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create components
    let storage = InMemoryStorage::new();
    let broker = InMemoryBroker::new();
    
    // Create and start scheduler
    let scheduler = SchedulerBuilder::new()
        .with_storage(storage)
        .with_broker(broker)
        .with_workers(4)
        .build();
    
    // Schedule a task
    let task = MyTask::new();
    let task_id = scheduler.schedule_delayed(task, Duration::from_secs(60)).await?;
    
    println!("Task scheduled: {}", task_id);
    
    // Run scheduler
    scheduler.run().await?;
    
    Ok(())
}
```

### Running Examples

```bash
# Simple example
cargo run --example simple

# Distributed example with Redis
cargo run --example distributed --features redis

# Custom configuration
cargo run -- --config ./config/scheduler.yaml
```

---

## Project Structure

```
scheduler-rs/
├── Cargo.toml              # Project manifest
├── Cargo.lock              # Dependency lock
├── config/
│   └── scheduler.yaml      # Configuration template
├── examples/
│   ├── simple.rs           # Basic usage
│   └── distributed.rs      # Multi-instance setup
├── src/
│   ├── lib.rs              # Library entry
│   ├── main.rs             # CLI entry (optional)
│   ├── config.rs           # Configuration types
│   ├── events/
│   │   └── mod.rs          # Event definitions
│   ├── brokers/
│   │   ├── mod.rs          # Broker module
│   │   ├── types.rs        # Broker trait
│   │   ├── memory.rs       # InMemoryBroker
│   │   └── redis.rs        # RedisBroker
│   ├── scheduler/
│   │   ├── mod.rs          # Scheduler module
│   │   ├── types.rs        # Scheduler trait
│   │   ├── runtime.rs      # Main event loop
│   │   ├── worker.rs       # Worker pool
│   │   └── cron.rs         # Cron scheduling
│   ├── storages/
│   │   ├── mod.rs          # Storage module
│   │   ├── types.rs        # Storage trait
│   │   ├── memory.rs       # InMemoryStorage
│   │   └── postgres.rs     # PostgreSQL storage
│   └── tasks/
│       ├── mod.rs          # Task module
│       ├── types.rs        # Task traits & states
│       └── utils.rs        # Task implementation
└── tests/
    ├── integration_tests.rs
    └── e2e_tests.rs
```

---

## Development Guidelines

### Code Style

- Follow Rust API guidelines
- Use `cargo fmt` for formatting
- Run `cargo clippy` before committing
- Add tests for new functionality

### Testing Strategy

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test component interactions
3. **E2E Tests**: Full scheduler scenarios
4. **Benchmarks**: Performance regression testing

### Commit Messages

Follow conventional commits:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `refactor:` Code refactoring
- `test:` Test changes
- `chore:` Maintenance

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.49 | Async runtime |
| serde | 1.0 | Serialization |
| anyhow | 1.0 | Error handling |
| thiserror | 2.0 | Custom errors |
| dashmap | 6.1 | Concurrent hashmap |
| parking_lot | 0.12 | Synchronization |
| time | 0.3 | Date/time handling |
| uuid | 1.19 | UUID generation |
| async-trait | 0.1 | Async traits |
| tracing | 0.1 | Logging |
| metrics | 0.22 | Metrics collection |

---

## Contributing

1. Check the roadmap for planned features
2. Open an issue to discuss major changes
3. Fork and create a feature branch
4. Write tests for your changes
5. Submit a pull request

---

## License

MIT License - see LICENSE file for details.

---

## Roadmap Checklist

Use this checklist to track progress:

### Phase 1: Core Infrastructure
- [ ] 1.1.1 Broker consume() method
- [ ] 1.1.2 Broker ack() method
- [ ] 1.1.3 Dead Letter Queue
- [ ] 1.2.1 InMemoryStorage
- [ ] 1.2.2 JSON persistence
- [ ] 1.2.3 Query methods
- [ ] 1.3.1 All state transitions
- [ ] 1.3.2 Retry configuration

### Phase 2: Scheduler Runtime
- [ ] 2.1.1 Event loop
- [ ] 2.1.2 Graceful shutdown
- [ ] 2.2.1 Worker pool
- [ ] 2.2.2 Task execution
- [ ] 2.2.3 Load balancing
- [ ] 2.3.1 Cron parser
- [ ] 2.3.2 Task schedulers
- [ ] 2.3.3 Scheduling API

### Phase 3: Event System
- [ ] 3.1 Event types defined
- [ ] 3.2.1 Event publisher
- [ ] 3.2.2 Event handlers
- [ ] 3.2.3 Event subscriptions
- [ ] 3.3.1 Event store

### Phase 4: Resilience & Observability
- [ ] 4.1.1 Circuit breaker
- [ ] 4.1.2 Backoff with jitter
- [ ] 4.1.3 Panic handling
- [ ] 4.2.1 Structured logging
- [ ] 4.2.2 Metrics collection
- [ ] 4.2.3 Health checks

### Phase 5: Production Features
- [ ] 5.1 Configuration file
- [ ] 5.2.1 Redis broker
- [ ] 5.2.2 PostgreSQL storage
- [ ] 5.2.3 Distributed locking
- [ ] 5.2.4 Leader election
- [ ] 5.3.1 Task priorities
- [ ] 5.3.2 Task dependencies
- [ ] 5.3.3 Task constraints

---

**Current Status**: Phase 1 in progress

**Last Updated**: 2024
