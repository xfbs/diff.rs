use crate::crates::{CrateInfo, CrateResponse, CrateSource, VersionInfo};
use crate::router::*;
use crate::components::*;
use implicit_clone::unsync::{IArray, IString};
use itertools::{Itertools, Position};
use log::*;
use similar::{ChangeTag, TextDiff};
use std::collections::BTreeSet;
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::suspense::*;
use yew_icons::{Icon as YewIcon, IconId};
use yewprint::id_tree::{InsertBehavior, Node, NodeId, TreeBuilder};
use yewprint::*;

#[derive(Clone, Debug)]
pub struct FileViewState {
    pub left: Arc<CrateSource>,
    pub right: Arc<CrateSource>,
    pub tree: TreeData<String>,
}

impl FileViewState {
    pub fn build(left: Arc<CrateSource>, right: Arc<CrateSource>) -> Self {
        let mut tree = TreeBuilder::new().build();
        let root = tree
            .insert(
                Node::new(NodeData {
                    data: "".into(),
                    ..Default::default()
                }),
                InsertBehavior::AsRoot,
            )
            .unwrap();

        // get common paths
        let paths: BTreeSet<String> = left
            .files
            .iter()
            .chain(right.files.iter())
            .map(|(path, _)| path)
            .cloned()
            .collect();

        for path in paths.into_iter() {
            let mut pos = root.clone();
            for segment in path.split("/").with_position() {
                match segment {
                    Position::First(s) | Position::Middle(s) => {
                        let existing = tree
                            .children_ids(&pos)
                            .unwrap()
                            .find(|i| tree.get(i).unwrap().data().data == s);
                        pos = if let Some(existing) = existing {
                            existing.clone()
                        } else {
                            tree.insert(
                                Node::new(NodeData {
                                    data: s.to_string(),
                                    label: s.into(),
                                    icon: Icon::FolderClose,
                                    has_caret: true,
                                    ..Default::default()
                                }),
                                InsertBehavior::UnderNode(&pos),
                            )
                            .unwrap()
                        };
                    }
                    Position::Last(s) | Position::Only(s) => {
                        pos = tree
                            .insert(
                                Node::new(NodeData {
                                    data: s.to_string(),
                                    label: s.into(),
                                    icon: Icon::Document,
                                    ..Default::default()
                                }),
                                InsertBehavior::UnderNode(&pos),
                            )
                            .unwrap();
                    }
                }
            }
        }

        let tree: TreeData<String> = tree.into();

        FileViewState {
            tree,
            left,
            right,
        }
    }
}

#[function_component]
pub fn FileView(props: &SourceViewProps) -> Html {
    let state = use_state(|| FileViewState::build(props.left.clone(), props.right.clone()));
    if state.left != props.left || state.right != props.right {
        state.set(FileViewState::build(props.left.clone(), props.right.clone()));
    }

    let tree: TreeData<String> = state.tree.clone();
    let state_clone = state.clone();
    let on_collapse = move |(node_id, _)| {
        let mut tree_clone: TreeData<String> = state_clone.tree.clone();
        let mut tree = tree_clone.borrow_mut();
        let node = tree.get_mut(&node_id).unwrap();
        let data = node.data_mut();
        data.is_expanded ^= true;
        data.icon = match data.is_expanded {
            true => Icon::FolderOpen,
            false => Icon::FolderClose,
        };
        state_clone.set((*state_clone).clone());
    };
    let on_expand = on_collapse.clone();
    let state_clone = state.clone();
    let onclick = move |(node_id, _)| {
        let mut tree_clone: TreeData<String> = state.tree.clone();
        let mut tree = tree_clone.borrow_mut();
        let node = tree.get_mut(&node_id).unwrap();
        let data = node.data_mut();
        data.is_selected ^= true;
        state.set((*state).clone());
    };
    html! {
        <Tree<String> {tree} {on_collapse} {on_expand} {onclick} />
    }
}

