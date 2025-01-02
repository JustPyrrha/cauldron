use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use toml::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrStringVec {
    String(String),
    Vec(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContributorsList {
    Plain(Vec<String>),
    WithRoles(HashMap<String, StringOrStringVec>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadataV0 {
    pub schema_version: u32,
    pub cauldron: PluginMetadataCauldron,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadataCauldron {
    pub id: String,
    pub version: String,
    pub metadata: Option<PluginMetadataCauldronMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadataCauldronMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub contributors: Option<ContributorsList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadataSchemaVersionOnly {
    pub schema_version: u32,
}