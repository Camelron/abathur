pub mod vm;

use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
struct CloudHypervisorFailed;

impl warp::reject::Reject for CloudHypervisorFailed {}

pub async fn start_vm(vm_args: abathur::StartVm, vm_handles: Arc<Mutex<HashMap<String, abathur::VmContext>>>) -> Result<impl warp::Reply, warp::Rejection> {
    // Start a VM
    println!("Starting VM: {}", vm_args.name);
    println!("Kernel: {}", vm_args.kernel);
    println!("Disks: {:?}", vm_args.disks);
    println!("CPUs: {}", vm_args.cpus);
    println!("Memory: {}", vm_args.memory);

    let guid = Uuid::new_v4().to_string();
    let api_socket = format!("/tmp/abathur/clh/{}.sock", guid);

    let clh_child = std::process::Command::new("cloud-hypervisor")
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
        .arg("--api-socket")
        .arg(api_socket.clone())
        .spawn();

    match clh_child {
        Ok(c) => {
            let handle = abathur::VmHandle {
                descriptor: vm_args.clone(),
                guid: guid.clone(),
                state: abathur::VmState::Starting,
            };
            
            let context = abathur::VmContext {
                handle: handle.clone(),
                api_socket,
            };
            vm_handles.lock().unwrap().insert(guid.clone(), context);

            let vm_handles_thread = vm_handles.clone();
            let guid_thread = guid.clone();
            let _ = std::thread::spawn(move || {
                vm::vm_main(c, guid_thread, vm_handles_thread);
            });
            println!("Started cloud-hypervisor for VM with GUID: {}", guid);
            Ok(warp::reply::json(&handle))
        }
        Err(e) => {
            eprintln!("Failed to start cloud-hypervisor: {}", e);
            Err(warp::reject::custom(CloudHypervisorFailed))
        }
    }
}

pub async fn list_vm(vm_handles: Arc<Mutex<HashMap<String, abathur::VmContext>>>) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Listing VMs");

    let handles = vm_handles.lock().unwrap();
    let handles: Vec<abathur::VmHandle> = handles.values().map(|c| c.handle.clone()).collect();
    Ok(warp::reply::json(&handles))
}


