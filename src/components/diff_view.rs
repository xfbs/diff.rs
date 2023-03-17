use super::*;
use crate::data::{CrateResponse, CrateSource, FileDiff, VersionDiff};
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

/// Contains information about contiguous changes
struct DiffGroupInfo {
    /// The actual changes
    group: Vec<(ChangeTag, bytes::Bytes)>,
    /// What range of lines the group covers (used as a Yew list key)
    range: std::ops::Range<usize>,
    /// Whether the group contains an actual diff (and therefore shows some context)
    in_context: bool,
}

#[derive(Properties, PartialEq, Clone)]
pub struct DiffViewProps {
    pub path: String,
    pub diff: Rc<VersionDiff>,
}

#[function_component]
pub fn DiffView(props: &DiffViewProps) -> Html {
    let empty = FileDiff::default();
    let file_diff = props.diff.files.get(&props.path).unwrap_or(&empty);

    // if this file does not exist, this will be none. so we use this trick to convert the none
    // case into an empty iterator, meaning that it will simply be rendered as an empty file.
    let mut changes = file_diff.changes.iter();
    let ranges = file_diff.context_ranges.iter();

    // Group contiguous lines by whether they contain an actual diff +/- some context buffer.
    let mut cursor = 0;
    let mut stack: Vec<DiffGroupInfo> = vec![];
    for next_range in ranges {
        // out of context lines
        if next_range.start != 0 {
            stack.push(DiffGroupInfo {
                group: changes
                    .by_ref()
                    .take(next_range.start - cursor)
                    .cloned()
                    .collect(),
                range: cursor..next_range.start,
                in_context: false,
            });
        }
        // in context lines
        stack.push(DiffGroupInfo {
            group: changes
                .by_ref()
                .take(next_range.end - next_range.start)
                .cloned()
                .collect(),
            range: next_range.clone(),
            in_context: true,
        });
        cursor = next_range.end;
    }
    if changes.len() > 0 {
        // Trailing unchanged lines at the end of a file
        stack.push(DiffGroupInfo {
            group: changes.by_ref().cloned().collect(),
            range: cursor..file_diff.changes.len(),
            in_context: false,
        });
    }

    let mut overall_index = 0;
    // Max of digits for a line number of this file
    let padding = file_diff.changes.len().max(1).to_string().len();

    html! {
        <pre>
        {
            stack.iter()
                .map(|DiffGroupInfo {group, range, in_context}| {
                    let res = html!{
                        <DiffLineGroup
                            key={format!("{:?}", range)}
                            group={group.clone()}
                            {in_context}
                            range={range.clone()}
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
    range: std::ops::Range<usize>,
    padding: usize,
}

#[function_component]
pub fn DiffLineGroup(props: &DiffLineGroupProps) -> Html {
    let full_range = props.range.clone();
    let range_to_hide = use_state(|| {
        if props.in_context {
            None
        } else {
            Some(full_range.clone())
        }
    });
    let onclick = {
        let range_to_hide = range_to_hide.clone();
        Callback::from(move |_| {
            range_to_hide.set(match *range_to_hide {
                Some(_) => None,
                None => Some(full_range.clone()),
            })
        })
    };
    let class = match (range_to_hide.as_ref(), props.in_context) {
        (Some(_), true) => "folded in-context",
        (Some(_), false) => "folded out-of-context",
        (None, true) => "in-context",
        (None, false) => "out-of-context",
    };

    let start_index = props.range.start;
    let end_index = props.range.end;
    let padding = props.padding;

    if range_to_hide.is_some() {
        html! {
            <button class={class} {onclick}>
                {format!("Show lines {start_index} to {end_index}")}
            </button>
        }
    } else {
        html! {
            <>
            if !props.in_context {
                <button class="folding-sticky" {onclick}>
                    {format!("Fold lines {start_index} to {end_index}")}
                </button>
            }
            <div class={class}>
            {
                props.group.iter().enumerate().map(|(index, (tag, change))| {
                    let overall_index = start_index + index;
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
