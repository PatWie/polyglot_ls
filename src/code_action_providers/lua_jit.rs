use std::ffi::CStr;

use mlua::{FromLua, Function, Lua, Result, Table, UserData, UserDataMethods, Value};
use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{Position, Range};
use tree_sitter::{
    ffi::{
        ts_node_child, ts_node_child_by_field_name, ts_node_child_count, ts_node_is_null,
        ts_node_named_child, ts_node_named_child_count, ts_node_next_sibling, ts_node_parent,
        ts_node_prev_sibling, ts_node_type, TSNode,
    },
    Node,
};

use crate::code_action_providers::parsed_document::ParsedDocument;

use super::helper;

#[derive(Copy, Clone, Debug)]
struct LuaNode(TSNode);

impl<'ts> From<LuaNode> for Node<'ts> {
    fn from(node: LuaNode) -> Self {
        unsafe {
            Node::from_raw(TSNode {
                context: node.0.context,
                id: node.0.id,
                tree: node.0.tree,
            })
        }
    }
}

impl<'ts> From<Node<'ts>> for LuaNode {
    fn from(node: Node<'ts>) -> Self {
        LuaNode(node.into_raw())
    }
}

impl UserData for LuaNode {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("kind", |_, node: &LuaNode, ()| {
            let node: TSNode = node.0;
            let s = unsafe { CStr::from_ptr(ts_node_type(node)) }
                .to_str()
                .unwrap();
            Result::Ok(s.to_string())
        });
        methods.add_method("child_count", |_, node: &LuaNode, ()| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_child_count(node) };
            Result::Ok(pnode)
        });
        methods.add_method("child", |_, node: &LuaNode, i: u32| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_child(node, i) };
            match unsafe { ts_node_is_null(pnode) } {
                true => Result::Ok(None),
                false => Result::Ok(Some(LuaNode(pnode))),
            }
        });
        methods.add_method("named_child_count", |_, node: &LuaNode, ()| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_named_child_count(node) };
            Result::Ok(pnode)
        });
        methods.add_method("named_child", |_, node: &LuaNode, i: u32| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_named_child(node, i) };
            match unsafe { ts_node_is_null(pnode) } {
                true => Result::Ok(None),
                false => Result::Ok(Some(LuaNode(pnode))),
            }
        });
        methods.add_method("parent", |_, node: &LuaNode, ()| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_parent(node) };
            match unsafe { ts_node_is_null(pnode) } {
                true => Result::Ok(None),
                false => Result::Ok(Some(LuaNode(pnode))),
            }
        });
        methods.add_method("prev_sibling", |_, node: &LuaNode, ()| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_prev_sibling(node) };
            match unsafe { ts_node_is_null(pnode) } {
                true => Result::Ok(None),
                false => Result::Ok(Some(LuaNode(pnode))),
            }
        });
        methods.add_method("range", |_, node: &LuaNode, ()| {
            let node: TSNode = node.0;
            let node: Node = unsafe { Node::from_raw(node) };
            let r: LuaRange = helper::ts_node_to_lsp_range(&node).into();
            Result::Ok(r)
        });
        methods.add_method("next_sibling", |_, node: &LuaNode, ()| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_next_sibling(node) };
            match unsafe { ts_node_is_null(pnode) } {
                true => Result::Ok(None),
                false => Result::Ok(Some(LuaNode(pnode))),
            }
        });
        methods.add_method("child_by_field_name", |_, node: &LuaNode, field_name: String| {
            let node: TSNode = node.0;
            let c_field_name = std::ffi::CString::new(field_name).unwrap();
            let child_node = unsafe {
                ts_node_child_by_field_name(
                    node,
                    c_field_name.as_ptr(),
                    c_field_name.as_bytes().len() as u32,
                )
            };

            match unsafe { ts_node_is_null(child_node) } {
                true => Result::Ok(None),
                false => Result::Ok(Some(LuaNode(child_node))),
            }
        });
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct LuaRange {
    pub start_line: u32,
    pub end_line: u32,
    pub start_character: u32,
    pub end_character: u32,
}
impl UserData for LuaRange {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("start_line", |_, this| Ok(this.start_line));
        fields.add_field_method_get("end_line", |_, this| Ok(this.end_line));
        fields.add_field_method_get("start_character", |_, this| Ok(this.start_character));
        fields.add_field_method_get("end_character", |_, this| Ok(this.end_character));
        fields.add_field_method_set("start_line", |_, this, val| {
            this.start_line = val;
            Ok(())
        });
        fields.add_field_method_set("end_line", |_, this, val| {
            this.end_line = val;
            Ok(())
        });
        fields.add_field_method_set("start_character", |_, this, val| {
            this.start_character = val;
            Ok(())
        });
        fields.add_field_method_set("end_character", |_, this, val| {
            this.end_character = val;
            Ok(())
        });
    }
}
impl FromLua for LuaRange {
    fn from_lua(value: Value, _: &Lua) -> Result<Self> {
        match value {
            Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            Value::Table(ud) => Ok(LuaRange {
                start_line: ud.get("start_line")?,
                end_line: ud.get("end_line")?,
                start_character: ud.get("start_character")?,
                end_character: ud.get("end_character")?,
            }),
            _ => unreachable!(),
        }
    }
}

impl From<LuaRange> for Range {
    fn from(value: LuaRange) -> Self {
        Self {
            start: Position {
                line: value.start_line,
                character: value.start_character,
            },
            end: Position {
                line: value.end_line,
                character: value.end_character,
            },
        }
    }
}
impl From<Range> for LuaRange {
    fn from(value: Range) -> Self {
        LuaRange {
            start_line: value.start.line,
            end_line: value.end.line,
            start_character: value.start.character,
            end_character: value.end.character,
        }
    }
}

impl FromLua for LuaNode {
    fn from_lua(value: Value, _: &Lua) -> Result<Self> {
        match value {
            Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => unreachable!(),
        }
    }
}

pub struct LuaDoc(ParsedDocument);

impl UserData for LuaDoc {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("uri", |_, active_doc: &LuaDoc, ()| -> Result<String> {
            Result::Ok(active_doc.0.uri.to_string())
        });
        methods.add_method("root", |_, doc: &LuaDoc, ()| -> Result<LuaNode> {
            Result::Ok(doc.0.tree.root_node().into())
        });
        methods.add_method(
            "query",
            |_, active_doc: &LuaDoc, (node, query): (LuaNode, String)| -> Result<Vec<LuaNode>> {
                let node: Node = node.into();
                let found_nodes = active_doc.0.query(&node, &query);
                let found_nodes: Vec<LuaNode> = found_nodes.into_iter().map(|n| n.into()).collect();

                Result::Ok(found_nodes)
            },
        );
        methods.add_method(
            "query_first",
            |_, active_doc: &LuaDoc, (node, query): (LuaNode, String)| -> Result<Option<LuaNode>> {
                let node: Node = node.into();
                let found_node = active_doc.0.find_first(&node, &query);
                match found_node {
                    Some(n) => Result::Ok(Some(n.into())),
                    None => Result::Ok(None),
                }
            },
        );
        methods.add_method(
            "node_from_range",
            |_, active_doc: &LuaDoc, range: LuaRange| -> Result<Option<LuaNode>> {
                let unwrapped_range: Range = range.into();
                let node = active_doc.0.get_ts_node_for_range(&unwrapped_range);
                Result::Ok(node.map(|n| n.into()))
            },
        );
        methods.add_method(
            "text_from_range",
            |_, active_doc: &LuaDoc, range: LuaRange| -> Result<String> {
                Result::Ok(active_doc.0.text_from_range(&range.into()))
            },
        );

        methods.add_method(
            "text_from_node",
            |_, active_doc: &LuaDoc, node: LuaNode| -> Result<String> {
                Result::Ok(active_doc.0.text_from_node(&node.into()))
            },
        );
    }
}

#[derive(Debug)]
struct LuaImpl {
    // Could be a functin at some point to dynamicall return an aciton name.
    action_name: String,
    is_triggered: Function,
    create_prompt: Function,
    placement_range: Function,
    process_answer: Option<Function>,
}

impl FromLua for LuaImpl {
    fn from_lua(value: Value, lua: &Lua) -> Result<Self> {
        let table: Table = lua.unpack(value)?;
        let action_name: String = table
            .get::<_, Function>("action_name")
            .expect("find action_name")
            .call(())
            .unwrap();

        let is_triggered = table.get("is_triggered").expect("find is_triggered");
        let create_prompt = table
            .get("create_prompt")
            .expect("create_prompt is_triggered");
        let placement_range = table
            .get("placement_range")
            .expect("findplacement_rangeis_triggered");
        let process_answer = table.get("process_answer").expect("find process_answer");
        Ok(LuaImpl {
            action_name,
            is_triggered,
            create_prompt,
            placement_range,
            process_answer,
            // lua,
        })
    }
}

pub struct LuaInterface {
    m: LuaImpl,
    // action_name: String,
    // is_triggered: Box<Function<'lua>>,
    // create_prompt: Function<'lua>,
    // placement_range: Function<'lua>,
    // process_answer: Option<Function<'lua>>,
    lua: Lua,
}

impl LuaInterface {
    pub fn new(lua_code: &str) -> Self {
        let lua = Lua::new();
        let value: LuaImpl = lua.load(lua_code).eval().unwrap();
        Self { m: value, lua: lua }
    }
    pub fn set_doc(&self, active_doc: ParsedDocument) {
        let active_doc = LuaDoc(active_doc);
        self.lua
            .globals()
            .set("active_doc", active_doc)
            .expect("can set active_doc");

        let helper_table = self.lua.create_table().unwrap();
        helper_table
            .set(
                "indent_text",
                self.lua
                    .create_function(|_, (text, indent_amount): (String, usize)| {
                        let indent = " ".repeat(indent_amount);
                        Ok(text
                            .lines()
                            .map(|line| format!("{}{}\n", indent, line))
                            .collect::<String>())
                    })
                    .unwrap(),
            )
            .unwrap();
        helper_table
            .set(
                "trim_suffix",
                self.lua
                    .create_function(|_, (text, suffix): (String, String)| {
                        let mut result = text.to_string();
                        if result.ends_with(&suffix) {
                            result.pop();
                        }
                        Ok(result)
                    })
                    .unwrap(),
            )
            .unwrap();
        self.lua.globals().set("helper", helper_table).unwrap();
    }

    pub fn is_triggered(&self, selection_range: &Range) -> bool {
        let selection_range: LuaRange = selection_range.to_owned().into();
        self.m
            .is_triggered
            .call(selection_range)
            .expect("can get result from lua function is_triggered")
    }

    pub fn build_prompt(&self, selection_range: &Range) -> Option<String> {
        let selection_range: LuaRange = selection_range.to_owned().into();
        self.m
            .create_prompt
            .call(selection_range)
            .expect("can get result from lua function create_prompt")
    }
    pub fn action_name(&self) -> String {
        self.m.action_name.clone()
    }
    pub fn process_answer(&self, text: &str, selection_range: &Range) -> Option<String> {
        let selection_range: LuaRange = selection_range.to_owned().into();
        match self.m.process_answer.as_ref() {
            Some(f) => f
                .call((text.to_string(), selection_range))
                .expect("can get result from lua function process_answer"),
            None => Some(text.to_owned()),
        }
    }
    pub fn placement_range(&self, selection_range: &Range) -> Option<Range> {
        let selection_range: LuaRange = selection_range.to_owned().into();

        let placement_range: Option<LuaRange> = self
            .m
            .placement_range
            .call(selection_range)
            .expect("can get result from lua function placement_range");

        let h: Option<Range> = placement_range.map(|n| n.into());
        //log::info!("Place at {:?}", placement_range);
        h
    }
}
