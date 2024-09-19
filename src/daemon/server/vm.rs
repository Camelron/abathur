use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

pub fn vm_main(mut clh: std::process::Child, guid: String, vm_handles: Arc<Mutex<HashMap<String, abathur::VmContext>>>) {
    println!("VM main started for GUID: {}", guid);
    {
        let map = vm_handles.lock().unwrap();
        if let Some(vm_context) = map.get(&guid) {
            // Use the reference to vm_context here
            println!("... for VM context {:?}", vm_context);
        } else {
            panic!("VM with key {} not found", &guid);
        }
    }
    
    let _ = clh.wait();
}



