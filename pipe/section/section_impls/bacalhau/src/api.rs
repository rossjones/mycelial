use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Deserialize)]
pub struct Job {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ID")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Namespace")]
    namespace: Option<String>,
    #[serde(rename = "Type")]
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Priority")]
    priority: Option<i64>,
    #[serde(rename = "Count")]
    count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Constraints")]
    constraints: Option<Vec<LabelSelectorRequirement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Meta")]
    meta: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Labels")]
    labels: Option<Map<String, Value>>,
    #[serde(rename = "Tasks")]
    tasks: Vec<Task>,
}

#[derive(Serialize, Deserialize)]
pub struct LabelSelectorRequirement {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Operator")]
    operator: String,
    #[serde(rename = "Values")]
    values: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Name")]
    name: Option<String>,

    #[serde(rename = "Engine")]
    engine: SpecConfig,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Publisher")]
    publisher: Option<SpecConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Env")]
    env: Option<Map<String, Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Meta")]
    meta: Option<Map<String, Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "InputSources")]
    input_sources: Option<Vec<InputSource>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ResultPaths")]
    result_paths: Option<Vec<ResultPath>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ResourcesConfig§")]
    resources_config: Option<ResourcesConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Network")]
    network: Option<NetworkConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "Timeouts")]
    timeouts: Option<TimeoutConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct InputSource {
    #[serde(rename = "Source")]
    source: SpecConfig,

    #[serde(rename = "rename")]
    rename: Option<String>,

    #[serde(rename = "Target")]
    target: String,
}

#[derive(Serialize, Deserialize)]
pub struct SpecConfig {
    #[serde(rename = "Type")]
    kind: String,

    #[serde(rename = "Params")]
    params: Map<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ResultPath {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Path")]
    path: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResourcesConfig {
    #[serde(rename = "CPU")]
    cpu: Option<String>,

    #[serde(rename = "Memory")]
    memory: Option<String>,

    #[serde(rename = "Disk")]
    disk: Option<String>,

    #[serde(rename = "GPU")]
    gpu: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(rename = "Type")]
    kind: String,
    #[serde(rename = "Domains")]
    domains: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct TimeoutConfig {
    #[serde(rename = "ExecutionTimeout")]
    execution_timeout: Option<i64>,
}
