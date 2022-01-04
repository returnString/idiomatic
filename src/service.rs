use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
	String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
	pub id: String,
	pub req: Option<IndexMap<String, Type>>,
	pub res: Option<IndexMap<String, Type>>,
	pub principal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
	pub id: String,
	#[serde(default)]
	pub principals: Vec<Principal>,
	#[serde(default)]
	pub endpoints: Vec<Endpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
	pub id: String,
	pub attributes: IndexMap<String, Type>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
	pub project_name: String,
	pub principals: Vec<Principal>,
}
