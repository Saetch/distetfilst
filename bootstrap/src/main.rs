use std::sync::Arc;
use axum::{
    routing::get,
    Router,
};
use axum::extract::{Path, State};
use axum::routing::post;
use tokio::sync::RwLock;


type SharedNodeVector = Arc<RwLock<Vec<String>>>;


#[derive(Debug, Clone)]
pub struct AppState {
    pub connected_nodes: SharedNodeVector,
    pub counter: Arc<RwLock<u64>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        connected_nodes: Arc::new(RwLock::new(  vec![])),
        counter: Arc::new(RwLock::new(0)),
    };
    // build our application with a single route
    let app = Router::new().route("/", get(hello_world)).route("/show_path", get(show_nodes)).route("/signup/{name}", post(signup)).route("/delete_node/{name}", post(delete_node)).with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world(State(state): State<AppState>) -> String {
    let mut lock = state.counter.write().await;
    *lock += 1;
    "Hello, World!\n".to_string() + &state.connected_nodes.read().await.join(", ") + &format!("\nCounter: {}", *lock)
}

async fn signup(State(state): State<AppState>, Path(name): Path<String>) -> String {
    let mut lock = state.connected_nodes.write().await;
    lock.push(name);
    lock.join(", ")
}

async fn delete_node(State(state): State<AppState>, Path(name): Path<String>) -> String {
    let mut lock = state.connected_nodes.write().await;
    lock.retain(|n| n != &name);
    lock.join(", ")
}

async fn show_nodes(State(state): State<AppState>) -> String {
    state.connected_nodes.read().await.join(", ")
}