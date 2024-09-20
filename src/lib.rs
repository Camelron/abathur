use std::sync::Mutex;
use std::sync::Arc;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct StartVm {
    pub name: String,
    pub kernel: String,
    pub disks: Vec<String>,
    pub cpus: String,
    pub memory: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct VmHandle {
    pub descriptor: StartVm,
    pub guid: String,
    pub state: VmState
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub enum VmState {
    Starting,
    Running,
    Stopped,
    Failed,
}

#[derive(Clone, Debug)]
pub struct VmContext {
    pub handle: VmHandle,
    pub api_socket: String,
    pub stdin: Arc<Mutex<std::process::ChildStdin>>,
    pub stdout: Arc<Mutex<std::process::ChildStdout>>,
}
