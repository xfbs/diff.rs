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
        <main>
            <nav id="files" aria-label="Files">
                <FileTree
                    diff={diff.clone()}
                    left={props.left.clone()}
                    right={props.right.clone()}
                    path={props.path.clone()}
                    {onselect}
                />
            </nav>
            <div id="diff-view">
                <DiffView {diff} path={props.path.clone()} />
            </div>
        </main>
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
    let ranges = props.diff.context_ranges.get(&props.path);

    // if this file does not exist, this will be none. so we use this trick to convert the none
    // case into an empty iterator, meaning that it will simply be rendered as an empty file.
    let mut changes = changes.iter().flat_map(|changes| changes.iter());
    let ranges = ranges.iter().flat_map(|changes| changes.iter());

    let mut cursor = 0;
    let mut stack: Vec<(bool, Vec<_>)> = vec![];
    for next_range in ranges {
        // out of context lines
        if next_range.start != 0 {
            stack.push((
                false,
                changes
                    .by_ref()
                    .take(next_range.start - cursor)
                    .cloned()
                    .collect(),
            ));
        }
        // in context lines
        stack.push((
            true,
            changes
                .by_ref()
                .take(next_range.end - next_range.start)
                .cloned()
                .collect(),
        ));
        cursor = next_range.end;
    }
    let mut overall_index = 0;
    // Max of digits for a line number of this file
    let padding = props
        .diff
        .files
        .get(&props.path)
        .map(|c| c.len().max(1))
        .unwrap_or(1)
        .ilog10() as usize;

    html! {
        <pre>
        {
            stack.iter()
                .map(|(in_context, group)| {
                    let res = html!{
                        <DiffLineGroup
                            group={group.clone()}
                            {in_context}
                            group_start_index={overall_index}
                            {padding}
                        />
                    };
                    overall_index += group.len();
                    res
                })
                .collect::<Html>()
        }
        </pre>
    }
}

#[derive(Properties, PartialEq)]
pub struct DiffLineGroupProps {
    group: Vec<(ChangeTag, bytes::Bytes)>,
    in_context: bool,
    group_start_index: usize,
    padding: usize,
}

#[function_component]
pub fn DiffLineGroup(props: &DiffLineGroupProps) -> Html {
    let folded = use_state(|| !props.in_context);
    let padding = props.padding;
    let onclick = {
        let folded = folded.clone();
        Callback::from(move |_| folded.set(!*folded))
    };
    let class = match (*folded, props.in_context) {
        (true, true) => "folded in-context",
        (true, false) => "folded out-of-context",
        (false, true) => "in-context",
        (false, false) => "out-of-context",
    };
    let group_start_index = props.group_start_index;
    let end_index = group_start_index + props.group.len();

    if *folded {
        html! {
            <button class={class} {onclick}>
                {format!("Show lines {group_start_index} to {end_index}")}
            </button>
        }
    } else {
        html! {
            <>
            if !props.in_context {
                <button class="folding-sticky" {onclick}>
                    {format!("Fold lines {group_start_index} to {end_index}")}
                </button>
            }
            <div class={class}>
            {
                props.group.iter().enumerate().map(|(index, (tag, change))| {
                    let overall_index = group_start_index + index;
                    let (sign, color) = match tag {
                        ChangeTag::Delete => ("-", "red"),
                        ChangeTag::Insert => ("+", "green"),
                        ChangeTag::Equal => (" ", "default"),
                    };
                    let contents = String::from_utf8_lossy(&change[..]);
                    html! {
                        <span style={format!("color: {color};")}>
                            {
                                format!(
                                    "{overall_index:>padding$} {sign} {}",
                                    contents
                                )
                            }
                        </span>
                    }
                }).collect::<Html>()
            }
            </div>
            </>
        }
    }
}
