use super::*;
use crate::data::{CrateResponse, CrateSource, VersionDiff};
use similar::ChangeTag;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Properties, PartialEq, Clone)]
pub struct SourceViewProps {
    pub info: Arc<CrateResponse>,
    pub left: Arc<CrateSource>,
    pub right: Arc<CrateSource>,
    pub path: String,
}

#[function_component]
pub fn SourceView(props: &SourceViewProps) -> Html {
    let diff = use_memo(
        |(left, right)| VersionDiff::new(left.clone(), right.clone()),
        (props.left.clone(), props.right.clone()),
    );
    let navigator = use_navigator().unwrap();
    let onselect = {
        let name = props.info.krate.id.clone();
        let left = props.left.version.num.clone();
        let right = props.right.version.num.clone();
        let navigator = navigator.clone();
        move |path: String| {
            navigator.push(&Route::File {
                name: name.clone(),
                left: left.clone(),
                right: right.clone(),
                path,
            })
        }
    };
    html! {
        <>
        <ComplexNavbar
            name={props.info.krate.id.clone()}
            left={props.left.version.num.clone()}
            right={props.right.version.num.clone()}
            info={props.info.clone()}
            onchange={
                let name = props.info.krate.id.clone();
                let path = props.path.clone();
                let navigator = navigator;
                move |(left, right)| {
                    navigator.push(&Route::File {
                        name: name.clone(),
                        left,
                        right,
                        path: path.clone(),
                    });
                }
            }
        />
        <Content>
        <div id="main">
            <div id="files">
                <FileTree
                    diff={diff.clone()}
                    left={props.left.clone()}
                    right={props.right.clone()}
                    path={props.path.clone()}
                    {onselect}
                />
            </div>
            <div id="diff-view">
                <DiffView {diff} path={props.path.clone()} />
            </div>
        </div>
        </Content>
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct DiffViewProps {
    pub path: String,
    pub diff: Rc<VersionDiff>,
}

#[function_component]
pub fn DiffView(props: &DiffViewProps) -> Html {
    let changes = props.diff.files.get(&props.path);

    // if this file does not exist, this will be none. so we use this trick to convert the none
    // case into an empty iterator, meaning that it will simply be rendered as an empty file.
    let changes = changes.iter().flat_map(|changes| changes.iter());
    html! {
        <pre>
        {
            changes
                .map(|(tag, change)| {
                    let (sign, color) = match tag {
                        ChangeTag::Delete => ("-", "red"),
                        ChangeTag::Insert => ("+", "green"),
                        ChangeTag::Equal => (" ", "default"),
                    };
                    html!{
                        <span style={format!("color: {color};")}>
                            { format!("{sign} {}", String::from_utf8_lossy(&change[..])) }
                        </span>
                    }
                })
                .collect::<Html>()
        }
        </pre>
    }
}
