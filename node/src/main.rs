
#[derive(Debug, Clone)]
struct AppState {
    pub name: String,
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

    if node_list.len() < 4 {

    }




    // --------------

    let state = AppState { name: host_name };
    let app = axum::Router::new().route("/", axum::routing::get(|| async move { state.name.clone() }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    // TODO: Acknowledge bootstrap container for successful connection
    //client.post(&bootstrap_target).send().await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
