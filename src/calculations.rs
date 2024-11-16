use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Calculation {
    pub name: String,
    pub wgsl_type: String,
}
