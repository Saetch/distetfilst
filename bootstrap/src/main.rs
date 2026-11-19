use std::sync::Arc;
use axum::{
    routing::get,
    Router,
};
use axum::extract::{Path, State};
use axum::routing::post;
use rustc_hash::FxHashSet;
use tokio::sync::RwLock;


type SharedNodeSet = Arc<RwLock<FxHashSet<String>>>;


#[derive(Debug, Clone)]
pub struct AppState {
    pub connected_nodes: SharedNodeSet,
    pub counter: Arc<RwLock<u64>>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        connected_nodes: Arc::new(RwLock::new(  FxHashSet::default() )),
        counter: Arc::new(RwLock::new(0)),
    };
    // build our application with a single route
    let app = Router::new().route("/", get(hello_world)).route("/signup/{name}", post(signup)).route("/delete_node/{name}", post(delete_node)).with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world(State(state): State<AppState>) -> String {
    let mut lock = state.counter.write().await;
    *lock += 1;
    let mut result = "".to_string();
    for node in state.connected_nodes.read().await.iter() {
        if !result.is_empty(){
            result.push_str(", ");
        }
        result.push_str(&node);
    }
    result
}

async fn signup(State(state): State<AppState>, Path(name): Path<String>) -> String {
    let mut lock = state.connected_nodes.write().await;
    lock.insert(name.clone());
    let mut result = "".to_string();
    //return all entries
    for item in lock.iter() {
        if !result.is_empty() {
            result.push_str(", ");
        }
        result.push_str(item);
    }
    result
}

async fn delete_node(State(state): State<AppState>, Path(name): Path<String>) -> String {
    let mut lock = state.connected_nodes.write().await;
    lock.retain(|n| n != &name);
    let mut result = "".to_string();
    for item in lock.iter() {
        if !result.is_empty() {
            result.push_str(", ");
        }
        result.push_str(item);
    }
    result
}