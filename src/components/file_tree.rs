use crate::data::{CrateSource, VersionDiff};
use anyhow::Result;
use camino::Utf8PathBuf;
use itertools::{Itertools, Position};
use std::{rc::Rc, sync::Arc};
use subslice_offset::SubsliceOffset;
use yew::prelude::*;
use yewprint::{
    id_tree::{InsertBehavior, Node, NodeId, TreeBuilder},
    *,
};

#[derive(Properties, PartialEq, Clone)]
pub struct FileTreeProps {
    pub old: Arc<CrateSource>,
    pub new: Arc<CrateSource>,
    pub diff: Rc<VersionDiff>,
    pub path: Utf8PathBuf,
    pub onselect: Callback<String>,
}

#[function_component]
pub fn FileTree(props: &FileTreeProps) -> Html {
    // use state: we build and cache a tree.
    let mut tree: TreeData<String> = use_memo(props.diff.clone(), |diff| build_tree(diff))
        .as_ref()
        .clone();

    // make current all files as deselected
    clear_selected(&mut tree).unwrap();

    // mark current file as selected and recursively expand parents
    mark_expand(&mut tree, &props.path.as_str()).unwrap();

    // on expand or collapse: toggle expanded bit of node.
    let update = use_force_update();
    let tree_clone: TreeData<String> = tree.clone();
    let on_collapse = move |(node_id, _)| {
        let mut tree_clone = tree_clone.clone();
        let mut tree = tree_clone.borrow_mut();
        let node = tree.get_mut(&node_id).unwrap();
        let data = node.data_mut();
        data.is_expanded ^= true;
        data.icon = match data.is_expanded {
            true => Icon::FolderOpen,
            false => Icon::FolderClose,
        };

        // need to use force update to trigger re-render because we used interior
        // mutability to update the data.
        update.force_update();
    };
    let on_expand = on_collapse.clone();

    // on click, we want to navigate to the selected file.
    let tree_clone: TreeData<String> = tree.clone();
    let callback = props.onselect.clone();
    let onclick = move |(node_id, _)| {
        let tree = tree_clone.borrow();
        let mut current = node_id;
        let mut path = vec![tree.get(&current).unwrap().data().data.clone()];
        while let Some(ancestor) = tree.ancestor_ids(&current).unwrap().next() {
            path.push(tree.get(ancestor).unwrap().data().data.clone());
            current = ancestor.clone();
        }
        path.pop();
        callback.emit(path.iter().rev().join("/"));
    };

    html! {
        <Tree<String> {tree} {on_collapse} {on_expand} {onclick} />
    }
}

fn build_tree(diff: &VersionDiff) -> TreeData<String> {
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

    for path in diff.files.keys() {
        let mut pos = root.clone();
        for (position, segment) in path.split('/').with_position() {
            let summary = {
                let end = path.subslice_offset(segment).unwrap() + segment.len();
                let path = &path[0..end];
                diff.summary.get(path).cloned().unwrap_or_default()
            };
            let summary_label = html! {
                <span style="white-space: nowrap;">
                    if summary.0 > 0 {
                        <span style="color: green;">{format!("+{}", summary.0)}</span>
                    }
                    {" "}
                    if summary.1 > 0 {
                        <span style="color: red;">{format!("-{}", summary.1)}</span>
                    }
                </span>
            };
            match position {
                Position::First | Position::Middle => {
                    let existing = tree
                        .children_ids(&pos)
                        .unwrap()
                        .find(|i| tree.get(i).unwrap().data().data == segment);
                    pos = if let Some(existing) = existing {
                        existing.clone()
                    } else {
                        tree.insert(
                            Node::new(NodeData {
                                data: segment.into(),
                                label: segment.into(),
                                icon: Icon::FolderClose,
                                has_caret: true,
                                secondary_label: Some(summary_label),
                                ..Default::default()
                            }),
                            InsertBehavior::UnderNode(&pos),
                        )
                        .unwrap()
                    };
                }
                Position::Last | Position::Only => {
                    pos = tree
                        .insert(
                            Node::new(NodeData {
                                data: segment.into(),
                                label: segment.into(),
                                icon: Icon::Document,
                                secondary_label: Some(summary_label),
                                ..Default::default()
                            }),
                            InsertBehavior::UnderNode(&pos),
                        )
                        .unwrap();
                }
            }
        }
    }

    tree.into()
}

fn clear_selected<T>(tree: &mut TreeData<T>) -> Result<()> {
    let mut tree = tree.borrow_mut();
    let node_ids: Vec<NodeId> = tree
        .traverse_post_order_ids(tree.root_node_id().unwrap())?
        .collect();
    for node_id in &node_ids {
        let node = tree.get_mut(node_id).unwrap();
        node.data_mut().is_selected = false;
    }
    Ok(())
}

fn mark_expand(tree: &mut TreeData<String>, path: &str) -> Result<()> {
    let mut tree = tree.borrow_mut();
    let mut current: NodeId = tree.root_node_id().unwrap().clone();

    for (position, segment) in path.split('/').with_position() {
        let result = tree
            .children_ids(&current)
            .unwrap()
            .find(|i| tree.get(i).unwrap().data().data == segment);
        current = match result {
            Some(id) => id.clone(),
            None => break,
        };
        let node = tree.get_mut(&current).unwrap();
        match position {
            Position::First | Position::Middle => {
                node.data_mut().is_expanded = true;
                node.data_mut().icon = Icon::FolderOpen;
            }
            Position::Last | Position::Only => {
                node.data_mut().is_selected = true;
            }
        }
    }

    Ok(())
}
