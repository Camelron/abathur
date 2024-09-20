pub mod vm;

use uuid::Uuid;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;
use std::sync::Mutex;
use std::process::{Stdio};
use warp::ws::{WebSocket, Message};
use abathur::VmContext;
use tokio::sync::mpsc;
use futures_util::{StreamExt, SinkExt};

#[derive(Debug)]
struct CloudHypervisorFailed;

impl warp::reject::Reject for CloudHypervisorFailed {}

pub async fn handle_socket(ws: WebSocket, guid: String, vm_handles: Arc<Mutex<HashMap<String, VmContext>>>) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();

    let vm_context = {
        let map = vm_handles.lock().unwrap();
        match map.get(&guid) {
            Some(context) => (*context).clone(),
            None => {
                eprintln!("VM with GUID {} not found", guid);
                return; // or handle the error appropriately
            }
        }
    };
    
    // Spawn a task to read from the VM's stdout and send to WebSocket
    tokio::spawn(async move {
        let mut stdout = vm_context.stdout.lock().unwrap();
        let mut buffer = [0; 1024];
        loop {
            let n = stdout.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            let msg = Message::binary(&buffer[..n]);
            tx.send(msg).unwrap();
        }
    });

    // Spawn a task to read from WebSocket and write to the VM's stdin
    tokio::spawn(async move {
        while let Some(msg) = ws_rx.next().await {
            match msg {
                Ok(msg) => {
                    if msg.is_close() {
                        break;
                    }
                    if msg.is_text() {
                        let text = msg.to_str().unwrap();
                        vm_context.stdin.lock().unwrap().write_all(text.as_bytes()).unwrap();
                    }
                }
                Err(e) => {
                    eprintln!("Websocket error: {}", e);
                    break;
                }
            }
            
        }
    });

    // Send messages from the VM's stdout to the WebSocket
    while let Some(msg) = rx.recv().await {
        ws_tx.send(msg).await.unwrap();
    }
}

pub async fn start_vm(vm_args: abathur::StartVm, vm_handles: Arc<Mutex<HashMap<String, VmContext>>>) -> Result<impl warp::Reply, warp::Rejection> {
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
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    match clh_child {
        Ok(mut c) => {
            let stdin = Arc::new(Mutex::new(c.stdin.take().unwrap()));
            let stdout = Arc::new(Mutex::new(c.stdout.take().unwrap()));

            let handle = abathur::VmHandle {
                descriptor: vm_args.clone(),
                guid: guid.clone(),
                state: abathur::VmState::Starting,
            };
            
            let context = abathur::VmContext {
                handle: handle.clone(),
                api_socket,
                stdin,
                stdout
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

pub async fn list_vm(vm_handles: Arc<Mutex<HashMap<String, VmContext>>>) -> Result<impl warp::Reply, warp::Rejection> {
    println!("Listing VMs");

    let handles = vm_handles.lock().unwrap();
    let handles: Vec<abathur::VmHandle> = handles.values().map(|c| c.handle.clone()).collect();
    Ok(warp::reply::json(&handles))
}


