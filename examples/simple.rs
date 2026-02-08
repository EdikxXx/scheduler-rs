use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::main;

type Task = Pin<Arc<dyn Future<Output = ()> + Send + Sync>>;

#[main]
async fn main() {}

async fn hello() {}
