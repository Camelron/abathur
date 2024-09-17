use clap::ArgMatches;
use warp::Filter;

pub async fn server_main(cmd_arguments: &ArgMatches) {
    let port = cmd_arguments.get_one::<String>("port").unwrap();
    let port: u16 = port.parse().expect("Port must be a number");

    println!("Starting daemon on port {}", port);

    let start_vm = warp::path("start_vm")
        .and(warp::post())
        .and(warp::body::json())
        .map(|vm: abathur::Vm| {
            println!("Starting VM: {}", vm.name);
            warp::reply::json(&vm)
        });

    let list_vm = warp::path("list_vm")
        .and(warp::get())
        .map(|| {
            println!("Listing VMs");
            warp::reply::json(&vec![abathur::Vm {
                name: "test".to_string(),
                kernel: "test".to_string(),
                disks: vec!["test".to_string()],
                cpus: "test".to_string(),
                memory: "test".to_string(),
            }])
        });

    let routes = start_vm.or(list_vm);

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
