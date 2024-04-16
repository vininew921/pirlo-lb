use std::env;

use actix_web::{get, http::header::LOCATION, web, App, HttpResponse, HttpServer, Responder};
use tokio::sync::Mutex;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

//Steps:
//1 - Create http server to listen to connections
//2 - Distribute requests to different api instances
//3 - Health check
//4 - profit
const INSTANCES: [u32; 3] = [7000, 7001, 7002];

struct LoadBalancerState {
    counter: Mutex<usize>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_env();

    let args: Vec<String> = env::args().collect();
    dbg!(args.clone());

    let port = &args[1];

    let state = web::Data::new(LoadBalancerState {
        counter: Mutex::new(0),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(distribute_handler)
    })
    .bind(("127.0.0.1", port.parse().unwrap()))?
    .run()
    .await
}

#[get("/")]
async fn distribute_handler(state: web::Data<LoadBalancerState>) -> impl Responder {
    let mut counter = state.counter.lock().await;
    *counter = if *counter + 1 == INSTANCES.len() {
        0
    } else {
        *counter + 1
    };

    tracing::info!("Redirecting to API at port {}", INSTANCES[*counter]);

    HttpResponse::Found()
        .insert_header((
            LOCATION,
            format!("http://localhost:{}", INSTANCES[*counter]),
        ))
        .finish()
}

fn init_env() {
    dotenvy::dotenv().ok();

    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    tracing_subscriber::registry()
        .with(stdout_log.with_filter(filter::LevelFilter::INFO))
        .init();
}
