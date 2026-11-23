use axum::extract::{Path, State};
use axum::routing::post;

#[derive(Debug, Clone)]
struct AppState {
    pub name: String,
    pub connected_nodes: Vec<String>,
}

#[tokio::main]
async fn main() {
    let host_name = std::env::var("HOSTNAME").unwrap();
    let bootstrap_target = std::env::var("BOOTSTRAP_TARGET").unwrap_or("http://bootstrap_container:3000/".to_string());

    println!("Host name: {}", host_name);
    println!("Bootstrap target: {}", bootstrap_target);
    let client = reqwest::Client::new();
    let bootstrap_target = bootstrap_target+"signup/"+&host_name;
    let mut response = client.post(&bootstrap_target).send().await.unwrap();
    while response.status() != reqwest::StatusCode::OK {
        response = client.post(&bootstrap_target).send().await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("Retrying connection to bootstrap container ...");
    }
    let text = response.text().await.unwrap();
    let node_list = text.split(", ").collect::<Vec<&str>>();
    let mut connected_nodes = Vec::new();

    if node_list.len() < 4 {
        for node in node_list {
            let establish_connection = String::from("http://") +node+"establish_connection/"+&host_name;
            let node_response = client.post(establish_connection).send().await.unwrap();
            if node_response.status() != reqwest::StatusCode::OK {
                println!("Failed to establish connection to node {}", node);
                continue;
            }
            connected_nodes.push(node.to_string());
        }
    }




    // --------------

    let state = AppState {
        name: host_name,
        connected_nodes: connected_nodes
    };
    let app = axum::Router::new()
        .route("/", axum::routing::get(|| async move { state.name.clone() }))
        .route("/establish_connection", post(establish_connection)).with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    // TODO: Acknowledge bootstrap container for successful connection
    //client.post(&bootstrap_target).send().await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn establish_connection(state: State<AppState>, Path(name): Path<String>) -> String {
    if state.connected_nodes.read().await.contains(&name) {
        state.connection_ongoing.store(false, std::sync::atomic::Ordering::SeqCst);
        return "success".to_string();
    }
    "failure".to_string()
}
