use crate::{
    components::{ComplexNavbar, Content, FileTree},
    data::{CrateResponse, CrateSource, FileDiff, VersionDiff},
    syntax::{highlight_changes, infer_syntax_for_file, syntect_style_to_css},
    version::VersionId,
    *,
};
use log::*;
use semver::Version;
use similar::ChangeTag;
use std::{rc::Rc, sync::Arc};
use syntect::highlighting::Style;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct SourceViewProps {
    pub src_info: Arc<CrateResponse>,
    pub dst_info: Arc<CrateResponse>,
    pub old: Arc<CrateSource>,
    pub new: Arc<CrateSource>,
    pub path: String,
}

#[function_component]
pub fn SourceView(props: &SourceViewProps) -> Html {
    let diff = use_memo((props.old.clone(), props.new.clone()), |(old, new)| {
        VersionDiff::new(old.clone(), new.clone())
    });
    let navigator = use_navigator().unwrap();
    let onselect = {
        let src_name = props.src_info.krate.id.clone();
        let dst_name = props.dst_info.krate.id.clone();
        let old: VersionId = props.old.version.num.clone().into();
        let new: VersionId = props.new.version.num.clone().into();
        let navigator = navigator.clone();
        move |path: String| {
            navigator.push(&Route::File {
                src_name: src_name.clone(),
                dst_name: dst_name.clone(),
                old: old.clone(),
                new: new.clone(),
                path,
            })
        }
    };
    html! {
        <>
        <ComplexNavbar
            src_name={props.src_info.krate.id.clone()}
            dst_name={props.dst_info.krate.id.clone()}
            old={props.old.version.num.clone()}
            new={props.new.version.num.clone()}
            src_info={props.src_info.clone()}
            dst_info={props.dst_info.clone()}
            onchange={
                let path = props.path.clone();
                let navigator = navigator;
                move |((src_name, old), (dst_name, new)): ((String, Version), (String, Version))| {
                    navigator.push(&Route::File {
                        src_name: src_name.clone(),
                        dst_name: dst_name.clone(),
                        old: old.clone().into(),
                        new: new.clone().into(),
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
                    old={props.old.clone()}
                    new={props.new.clone()}
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
    group: Vec<(ChangeTag, Vec<(Style, bytes::Bytes)>)>,
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
    let is_identical_version = props.diff.left.version == props.diff.right.version;

    // Determine which syntax should be used for this file. It will be based
    // first on the file's name, then the file's extension, then the first line.
    let syntax = infer_syntax_for_file(
        &props.path,
        file_diff
            .changes
            .iter()
            .find(|(tag, _)| *tag != ChangeTag::Delete)
            .and_then(|(_, line)| std::str::from_utf8(line).ok()),
    );
    info!("Highlighting {} as {}", syntax.name, props.path);

    // Apply highlighting to every change in the file.
    let mut changes = highlight_changes(syntax, &file_diff.changes).into_iter();
    let ranges = file_diff.context_ranges.iter();

    // Group contiguous lines by whether they contain an actual diff +/- some context buffer.
    let mut cursor = 0;
    let mut stack: Vec<DiffGroupInfo> = vec![];
    for next_range in ranges {
        // out of context lines
        if next_range.start != 0 {
            stack.push(DiffGroupInfo {
                group: changes.by_ref().take(next_range.start - cursor).collect(),
                range: cursor..next_range.start,
                in_context: false,
            });
        }
        // in context lines
        stack.push(DiffGroupInfo {
            group: changes
                .by_ref()
                .take(next_range.end - next_range.start)
                .collect(),
            range: next_range.clone(),
            in_context: true,
        });
        cursor = next_range.end;
    }
    if changes.len() > 0 {
        // Trailing unchanged lines at the end of a file
        stack.push(DiffGroupInfo {
            group: changes.by_ref().collect(),
            range: cursor..file_diff.changes.len(),
            // When comparing a version of the crate to itself, this group will
            // always contain the full text of the file. Don't collapse it.
            in_context: is_identical_version,
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
    group: Vec<(ChangeTag, Vec<(Style, bytes::Bytes)>)>,
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
                    let (sign, bg_color) = match tag {
                        ChangeTag::Delete => ("-", "#ffebe9"),
                        ChangeTag::Insert => ("+", "#dafbe1"),
                        ChangeTag::Equal => (" ", "default"),
                    };
                    html! {
                        <div style={format!("background-color:{bg_color}")}>
                            {
                                format!("{overall_index:>padding$} {sign} ")
                            }
                            {
                                change.iter().map(|(style, text)| {
                                    let style = syntect_style_to_css(style);
                                    let contents = String::from_utf8_lossy(&text[..]);
                                    html! {
                                        <span style={style}>{contents}</span>
                                    }
                                })
                                .collect::<Html>()
                            }
                        </div>
                    }
                }).collect::<Html>()
            }
            </div>
            </>
        }
    }
}
