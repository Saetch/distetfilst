use axum::{
    routing::get,
    Router,
};
use axum::extract::State;

#[derive(Debug, Clone)]
pub struct AppState {
    pub connected_nodes: Vec<String>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        connected_nodes: vec!["asdfasdf".to_string(), "asdfasdf".to_string(),"uro2hgo".to_string()],
    };
    // build our application with a single route
    let app = Router::new().route("/", get(hello_world)).with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world(State(state): State<AppState>) -> String {
    "Hello, World!\n".to_string() + &state.connected_nodes.join(", ")
}