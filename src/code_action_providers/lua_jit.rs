use std::ffi::CStr;

use mlua::{
    FromLua, Function, Lua, Result, UserData, UserDataMethods,
    Value::{self},
};
use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{Position, Range};
use tree_sitter::{
    ffi::{
        ts_node_is_null, ts_node_next_sibling, ts_node_parent, ts_node_prev_sibling, ts_node_type,
        TSNode,
    },
    Node,
};

use crate::code_action_providers::parsed_document::ParsedDocument;

use super::helper;

#[derive(Copy, Clone, Debug)]
struct WrappedNode(TSNode);

impl UserData for WrappedNode {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("kind", |_, node: &WrappedNode, ()| {
            let node: TSNode = node.0;
            let s = unsafe { CStr::from_ptr(ts_node_type(node)) }
                .to_str()
                .unwrap();
            Result::Ok(s.to_string())
        });
        methods.add_method("parent", |_, node: &WrappedNode, ()| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_parent(node) };
            match unsafe { ts_node_is_null(pnode) } {
                true => Result::Ok(None),
                false => Result::Ok(Some(WrappedNode(pnode))),
            }
        });
        methods.add_method("prev", |_, node: &WrappedNode, ()| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_prev_sibling(node) };
            match unsafe { ts_node_is_null(pnode) } {
                true => Result::Ok(None),
                false => Result::Ok(Some(WrappedNode(pnode))),
            }
        });
        methods.add_method("range", |_, node: &WrappedNode, ()| {
            let node: TSNode = node.0;
            let node: Node = unsafe { Node::from_raw(node) };
            let r: WrappedRange = helper::ts_node_to_lsp_range(&node).into();
            Result::Ok(r)
        });
        methods.add_method("next", |_, node: &WrappedNode, ()| {
            let node: TSNode = node.0;
            let pnode = unsafe { ts_node_next_sibling(node) };
            match unsafe { ts_node_is_null(pnode) } {
                true => Result::Ok(None),
                false => Result::Ok(Some(WrappedNode(pnode))),
            }
        });
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct WrappedRange {
    pub start_line: u32,
    pub end_line: u32,
    pub start_character: u32,
    pub end_character: u32,
}
impl UserData for WrappedRange {
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
impl FromLua<'_> for WrappedRange {
    fn from_lua(value: Value, _: &Lua) -> Result<Self> {
        print!("gottach {:?}", value);
        match value {
            Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            Value::Table(ud) => Ok(WrappedRange {
                start_line: ud.get("start_line")?,
                end_line: ud.get("end_line")?,
                start_character: ud.get("start_character")?,
                end_character: ud.get("end_character")?,
            }),
            _ => unreachable!(),
        }
    }
}

impl From<WrappedRange> for Range {
    fn from(value: WrappedRange) -> Self {
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
impl From<Range> for WrappedRange {
    fn from(value: Range) -> Self {
        WrappedRange {
            start_line: value.start.line,
            end_line: value.end.line,
            start_character: value.start.character,
            end_character: value.end.character,
        }
    }
}

impl FromLua<'_> for WrappedNode {
    fn from_lua(value: Value, _: &Lua) -> Result<Self> {
        match value {
            Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => unreachable!(),
        }
    }
}

pub struct WrappedDoc {
    pub parsed_doc: ParsedDocument,
}

impl UserData for WrappedDoc {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_uri", |_, doc: &WrappedDoc, ()| {
            Result::Ok(doc.parsed_doc.uri.to_string())
        });
        methods.add_method("get_root", |_, doc: &WrappedDoc, ()| {
            let node: WrappedNode = WrappedNode(doc.parsed_doc.tree.root_node().into_raw());
            Result::Ok(node)
        });
        methods.add_method(
            "query_first",
            |_, doc: &WrappedDoc, (node, query): (WrappedNode, String)| {
                let node: Node = unsafe {
                    Node::from_raw(TSNode {
                        context: node.0.context,
                        id: node.0.id,
                        tree: node.0.tree,
                    })
                };

                let found_node = doc.parsed_doc.find_first(&node, &query);
                if let Some(node) = found_node {
                    let node: WrappedNode = WrappedNode(node.into_raw());
                    return Result::Ok(Some(node));
                }
                return Result::Ok(None);
            },
        );

        methods.add_method("get_node", |_, doc: &WrappedDoc, range: WrappedRange| {
            let unwrapped_range: Range = range.into();
            let node = doc.parsed_doc.get_ts_node_for_range(&unwrapped_range);
            let wrapped_node = node.map(|n| {
                let node: WrappedNode = WrappedNode(n.into_raw());
                node
            });
            Result::Ok(wrapped_node)
        });

        // methods.add_method("get_range", |_, doc: &WrappedDoc, node: WrappedNode| {
        //     let node: Node = unsafe {
        //         Node::from_raw(TSNode {
        //             context: node.0.context,
        //             id: node.0.id,
        //             tree: node.0.tree,
        //         })
        //     };
        //     let r: WrappedRange = helper::ts_node_to_lsp_range(&node).into();
        //
        //     Result::Ok(r)
        // });

        methods.add_method("get_text", |_, doc: &WrappedDoc, node: WrappedNode| {
            let node: Node = unsafe {
                Node::from_raw(TSNode {
                    context: node.0.context,
                    id: node.0.id,
                    tree: node.0.tree,
                })
            };

            Result::Ok(doc.parsed_doc.get_text(&node))
        });
    }
}

pub struct LuaInterface {
    pub lua: Lua,
}

impl LuaInterface {
    pub fn new(lua_code: &str) -> Self {
        let lua = Lua::new();
        lua.load(lua_code).exec().unwrap();

        Self { lua }
    }
    pub fn set_doc(&self, doc: ParsedDocument) {
        let safe_doc = WrappedDoc { parsed_doc: doc };
        self.lua
            .globals()
            .set("doc", safe_doc)
            .expect("can set doc");

        let t = self.lua.create_table().unwrap();
        t.set(
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
        );
        t.set(
            "trim_suffix",
            self.lua
                .create_function(|_, (text, rune): (String, String)| {
                    let mut result = text.to_string();
                    if result.ends_with(&rune) {
                        result.pop();
                    }
                    Ok(result)
                })
                .unwrap(),
        );
        self.lua.globals().set("helper", t);
    }

    pub fn is_triggered(&self, start_range: &Range) -> bool {
        let start_range: WrappedRange = start_range.to_owned().into();
        let lua_fn: Function = self
            .lua
            .globals()
            .get("is_triggered")
            .expect("can find lua function is_triggered");
        lua_fn
            .call(start_range)
            .expect("can get result from lua function is_triggered")
    }

    pub fn build_prompt(&self, start_range: &Range) -> Option<String> {
        let lua_fn: Function = self
            .lua
            .globals()
            .get("create_prompt")
            .expect("can find lua function create_prompt");
        let start_range: WrappedRange = start_range.to_owned().into();
        lua_fn
            .call(start_range)
            .expect("can get result from lua function create_prompt")
    }
    pub fn action_name(&self) -> Option<String> {
        let lua_fn: Function = self
            .lua
            .globals()
            .get("action_name")
            .expect("can find lua function action_name");
        lua_fn
            .call(())
            .expect("can get result from lua function process_answer")
    }
    pub fn process_answer(&self, text: &str, start_range: &Range) -> Option<String> {
        let lua_fn: Function = self
            .lua
            .globals()
            .get("process_answer")
            .expect("can find lua function process_answer");
        let start_range: WrappedRange = start_range.to_owned().into();
        lua_fn
            .call((text.to_string(), start_range))
            .expect("can get result from lua function process_answer")
    }
    pub fn placement_range(&self, start_range: &Range) -> Option<Range> {
        let lua_fn: Function = self
            .lua
            .globals()
            .get("placement_range")
            .expect("can find lua function placement_range");

        let start_range: WrappedRange = start_range.to_owned().into();

        let wrange: Option<WrappedRange> = lua_fn
            .call(start_range)
            .expect("can get result from lua function placement_range");

        let h: Option<Range> = wrange.map(|n| n.into());
        h
    }
}
