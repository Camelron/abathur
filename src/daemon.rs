pub mod server;

use clap::ArgMatches;
use warp::Filter;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use warp::ws::Ws;

pub async fn daemon_main(cmd_arguments: &ArgMatches) {
    let port = cmd_arguments.get_one::<String>("port").unwrap();
    let port: u16 = port.parse().expect("Port must be a number");

    println!("Starting daemon on port {}", port);

    let vm_handles = Arc::new(Mutex::new(HashMap::new()));
    let vm_handles_filter = warp::any().map(move || Arc::clone(&vm_handles));

    let start_vm_endpoint = warp::post()
        .and(warp::path("start_vm"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(vm_handles_filter.clone())
        .and_then(server::start_vm);

    let list_vm_endpoint = warp::get()
        .and(warp::path("list_vm"))
        .and(warp::path::end())
        .and(vm_handles_filter.clone())
        .and_then(server::list_vm);

    let connect_vm_endpoint = warp::path("connect_vm")
        .and(warp::path::param())
        .and(warp::ws())
        .and(vm_handles_filter.clone())
        .map(|guid: String, ws: Ws, vm_handles: Arc<Mutex<HashMap<String, abathur::VmContext>>>| {
            ws.on_upgrade(move |socket| server::handle_socket(socket, guid, vm_handles))
        });

    let routes = start_vm_endpoint
        .or(list_vm_endpoint)
        .or(connect_vm_endpoint);

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
