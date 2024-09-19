
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

#[derive(Debug)]
pub struct VmContext {
    pub handle: VmHandle,
    pub api_socket: String
}
