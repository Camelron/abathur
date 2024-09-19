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
        Ok(c) => {
            let guid = Uuid::new_v4().to_string();
            let handle = abathur::VmHandle {
                descriptor: vm_args.clone(),
                guid: guid.clone(),
            };

            let ret = handle.clone();
            let vm_handles_thread = vm_handles.clone();
            let guid_thread = guid.clone();
            let thread_handle = std::thread::spawn(move || {
                vm::vm_main(c, guid_thread, vm_handles_thread);
            });

            let context = abathur::VmContext {
                handle: handle,
                process: thread_handle,
                state: abathur::VmState::Starting,
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

pub async fn list_vm(vm_handles: Arc<Mutex<HashMap<String, abathur::VmContext>>>) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Listing VMs");

    let handles = vm_handles.lock().unwrap();
    let handles: Vec<abathur::VmHandle> = handles.values().map(|c| c.handle.clone()).collect();
    return Ok(warp::reply::json(&handles));
}


