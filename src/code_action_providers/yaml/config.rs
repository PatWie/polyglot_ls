use serde::{Deserialize, Serialize};
use std::fs;
use tree_sitter::Node;

use crate::code_action_providers::helper::findup;

#[derive(Debug, Deserialize, Serialize)]
pub struct CodeActionConfig {
    pub code_actions: Vec<CodeAction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodeAction {
    /// The name of the code action.
    pub name: String,
    /// The triggers that activate this code action.
    pub triggers: Vec<Trigger>,
    /// The context in which this code action is applicable.
    pub context: Context,
    /// The placement strategies that determine where the result is displayed.
    pub placement_strategies: Vec<PlacementStrategy>,
    /// The template used to generate the prompt for this code action.
    pub prompt_template: String,
    /// The template used to embed the answer for this code action, if applicable.
    pub answer_template: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trigger {
    kind: String,
    relation: Relation,
}

impl Trigger {
    pub fn is_triggered(&self, start_node: Option<Node>) -> bool {
        if let Some(node) = start_node {
            match self.relation {
                Relation::Findup => findup(Some(node), &self.kind).is_some(),
                Relation::Exact => node.kind() == self.kind,
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Context {
    kind: String,
    relation: Relation,
    pub hints: Vec<Hint>,
}
impl Context {
    pub fn find<'a>(&self, start_node: Option<Node<'a>>) -> Option<Node<'a>> {
        if let Some(node) = start_node {
            match self.relation {
                Relation::Findup => findup(Some(node), &self.kind),
                Relation::Exact => {
                    if node.kind() == self.kind {
                        start_node
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Hint {
    pub name: String,
    pub query: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlacementStrategy {
    pub query: String,
    pub position: Position,
}

#[derive(Debug, Deserialize, Serialize)]
enum Relation {
    #[serde(rename = "findup")]
    Findup,
    #[serde(rename = "exact")]
    Exact,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    #[serde(rename = "replace_block")]
    ReplaceBlock,
    #[serde(rename = "replace_exact")]
    ReplaceExact,
    #[serde(rename = "before")]
    Before,
}

impl CodeActionConfig {
    pub fn from_yaml<A: AsRef<std::path::Path>>(
        path: &A,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let yaml_content = fs::read_to_string(path)?;
        let config: CodeActionConfig = serde_yaml::from_str(&yaml_content)?;
        Ok(config)
    }
}
