
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Vm {
    pub name: String,
    pub kernel: String,
    pub disks: Vec<String>,
    pub cpus: String,
    pub memory: String,
}
