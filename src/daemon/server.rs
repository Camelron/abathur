use clap::ArgMatches;
use warp::Filter;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
struct CloudHypervisorFailed;

impl warp::reject::Reject for CloudHypervisorFailed {}

async fn start_vm(vm_args: abathur::StartVm, vm_handles: Arc<Mutex<HashMap<String, abathur::VmContext>>>) -> Result<impl warp::Reply, warp::Rejection> {
    // Start a VM
    println!("Starting VM: {}", vm_args.name);
    println!("Kernel: {}", vm_args.kernel);
    println!("Disks: {:?}", vm_args.disks);
    println!("CPUs: {}", vm_args.cpus);
    println!("Memory: {}", vm_args.memory);

    let clh_command = std::process::Command::new("cloud-hypervisor")
        .env("PATH", "/bin")
        // .arg("-v")
        .arg("--kernel")
        .arg(vm_args.kernel.clone())
        .arg("--disk")
        .args(vm_args.disks.iter().map(|d| format!("path={}", d)))
        .arg("--cpus")
        .arg(format!("boot={}", vm_args.cpus.clone()))
        .arg("--memory")
        .arg(format!("size={}", vm_args.memory.clone()))
        .spawn();

    match clh_command {
        Ok(mut c) => {
            let guid = Uuid::new_v4().to_string();
            let handle = abathur::VmHandle {
                descriptor: vm_args.clone(),
                guid: guid.clone(),
            };

            let ret = handle.clone();
            let thread_handle = std::thread::spawn(move || {
                let _ = c.wait();
            });

            let context = abathur::VmContext {
                handle: handle,
                process: thread_handle,
            };
            
            vm_handles.lock().unwrap().insert(guid.clone(), context);
            println!("Started cloud-hypervisor for VM with GUID: {}", guid);
            return Ok(warp::reply::json(&ret));
        }
        Err(e) => {
            eprintln!("Failed to start cloud-hypervisor: {}", e);
            return Err(warp::reject::custom(CloudHypervisorFailed));
        }
    };
}

pub async fn server_main(cmd_arguments: &ArgMatches) {
    let port = cmd_arguments.get_one::<String>("port").unwrap();
    let port: u16 = port.parse().expect("Port must be a number");

    println!("Starting daemon on port {}", port);

    let vm_handles = Arc::new(Mutex::new(HashMap::new()));
    let vm_handles_filter = warp::any().map(move || Arc::clone(&vm_handles));

    // let start_vm = warp::path("start_vm")
    //     .and(warp::post())
    //     .and(warp::body::json())
    //     .map(|vm: abathur::StartVm| {
    //         start_vm(&vm);
    //         warp::reply::json(&vm)
    //     });

    let start_vm = warp::post()
        .and(warp::path("start_vm"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(vm_handles_filter.clone())
        .and_then(start_vm);

    let list_vm = warp::path("list_vm")
        .and(warp::get())
        .map(|| {
            println!("Listing VMs");
            warp::reply::json(&vec![abathur::StartVm {
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
