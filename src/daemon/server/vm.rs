use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

pub fn vm_main(mut clh: std::process::Child, guid: String, vm_handles: Arc<Mutex<HashMap<String, abathur::VmContext>>>) {
    let _ = clh.wait();
}



