use super::*;
use crate::data::{CrateResponse, CrateSource};
use similar::{ChangeTag, TextDiff};
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
    let left = props
        .left
        .files
        .get(&props.path)
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let right = props
        .right
        .files
        .get(&props.path)
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();
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
            versions={props.info.versions.iter().map(|v| v.num.clone()).collect::<Vec<_>>()}
            onchange={
                let name = props.info.krate.id.clone();
                let path = props.path.clone();
                let navigator = navigator.clone();
                move |(left, right)| {
                    navigator.push(&Route::File {
                        name: name.clone(),
                        left: left,
                        right: right,
                        path: path.clone(),
                    });
                }
            }
        />
        <Content>
        <div style="display: flex;">
            <div style="width: 300px;">
                <FileTree
                    info={props.info.clone()}
                    left={props.left.clone()}
                    right={props.right.clone()}
                    path={props.path.clone()}
                    {onselect}
                />
            </div>
            <div style="width: 50%; padding-left: 8px;">
                <DiffView {left} {right} path={props.path.clone()} />
            </div>
        </div>
        </Content>
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct DiffViewProps {
    pub path: String,
    pub left: String,
    pub right: String,
}

#[function_component]
pub fn DiffView(props: &DiffViewProps) -> Html {
    let diff = TextDiff::from_lines(&props.left, &props.right);
    html! {
        <>
        <pre>
        {
            diff.iter_all_changes().map(|change| {
                let (sign, color) = match change.tag() {
                    ChangeTag::Delete => ("-", "red"),
                    ChangeTag::Insert => ("+", "green"),
                    ChangeTag::Equal => (" ", "default"),
                };
                html!{
                    <span style={format!("color: {color};")}>
                        { format!("{sign} {change}") }
                    </span>
                }
            }).collect::<Html>()
        }
        </pre>
        </>
    }
}
