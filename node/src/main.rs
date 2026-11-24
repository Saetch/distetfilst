use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use tokio::sync::RwLock;
use rand::random;

type SharedStringVector = Arc<RwLock<Vec<String>>>;
#[derive(Debug, Clone)]
struct AppState {
    pub name: String,
    pub connected_nodes: SharedStringVector,
    pub nats_client: async_nats::Client,
    pub connection_allowed: Arc<RwLock<bool>>,
    pub http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    let host_name = std::env::var("HOSTNAME").unwrap();
    let bootstrap_target = std::env::var("BOOTSTRAP_TARGET").unwrap_or("http://bootstrap_container:3000/".to_string());
    let nats_target = std::env::var("NATS_TARGET").unwrap_or("nats-seed:4222".to_string());
    println!("Host name: {}", host_name);
    println!("Bootstrap target: {}", bootstrap_target);
    let client = reqwest::Client::new();
    let bootstrap_target_signup = bootstrap_target.clone()+"signup/"+&host_name;
    let mut response = client.post(&bootstrap_target_signup).send().await.unwrap();
    while response.status() != reqwest::StatusCode::OK {
        response = client.post(&bootstrap_target_signup).send().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("Retrying connection to bootstrap container ...");
    }
    let text = response.text().await.unwrap();
    let node_list = text.split(", ").collect::<Vec<&str>>();
    let mut connected_nodes = Vec::new();

    if node_list.len() < 5 {
        for node in node_list {
            if node == host_name { continue; }
            let establish_connection = String::from("http://") +node+":3001/establish_connection/"+&host_name;
            let node_response = client.post(establish_connection).send().await.unwrap();
            if node_response.status() != reqwest::StatusCode::OK {
                println!("Failed to establish connection to node {}", node);
                continue;
            }
            connected_nodes.push(node.to_string());
        }
    }




    // --------------
    let mut nats_client = async_nats::connect(&nats_target).await;
    while nats_client.is_err() {
        println!("Failed to connect to nats {}, retrying ... ", nats_target);
        nats_client = async_nats::connect(&nats_target).await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    let nats_client = nats_client.unwrap();
    let state = AppState {
        name: host_name.clone(),
        connected_nodes: Arc::new(RwLock::new(connected_nodes)),
        nats_client,
        connection_allowed: Arc::new(RwLock::new(true)),
        http_client: client.clone(),
    };
    println!("{:?}",state.connected_nodes.read().await);
    let hn_clone = host_name.clone();
    let app = axum::Router::new()
        .route("/", get(|| async move { hn_clone }))
        .route("/establish_connection/{name}", post(establish_connection))
        .route("/get_connections", get(get_connections))
        .route("/is_connection_allowed", get(is_connection_allowed))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    client.post(bootstrap_target+"connection_completed/"+ &host_name).send().await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn establish_connection(state: State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    let allowed_lock = state.connection_allowed.read().await;
    if !*allowed_lock {
        let response = Response::builder().status(StatusCode::CONFLICT).body(Body::from("No connection allowed!")).unwrap();
        return response;
    }
    drop(allowed_lock);
    let mut lock = state.connected_nodes.write().await;
    if lock.contains(&name) {
        println!("Already established connection to node {}", name);
        return Response::builder().status(StatusCode::CONFLICT).body(Body::from("Already established connection to node !")).unwrap();
    }
    println!("Connection to node {} established", &name);
    if lock.len() > 2 {
        let mut node_to_disconnect = random::<usize>() % lock.len();
        if lock[node_to_disconnect] == name { node_to_disconnect += 1; };
        if node_to_disconnect==lock.len() {
            node_to_disconnect = 0;
        }
        println!("Disconnecting node {} from node {}", &lock[node_to_disconnect], &name);
        state.http_client.post("http://".to_string() + &lock[node_to_disconnect]+":3001/disconnect_from/"+&state.name).send().await.unwrap();
    }
    lock.push(name);
    Response::builder().status(StatusCode::OK).body(Body::empty()).unwrap()
}

async fn get_connections(state: State<AppState>) -> impl IntoResponse {
    let lock = state.connected_nodes.read().await;
    println!("Current connections: {:?}", *lock);
    lock.join(", ")
}

async fn is_connection_allowed(state: State<AppState>) -> impl IntoResponse {
    let lock = state.connection_allowed.read().await;
    format!("{}\n", *lock)
}
