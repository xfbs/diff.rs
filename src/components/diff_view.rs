use crate::{
    data::{FileDiff, VersionDiff},
    syntax::{highlight_changes, infer_syntax_for_file, syntect_style_to_css},
};
use camino::Utf8PathBuf;
use log::*;
use similar::ChangeTag;
use std::rc::Rc;
use syntect::highlighting::Style;
use yew::prelude::*;

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
    pub path: Utf8PathBuf,
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
        props.path.as_str(),
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
        <div class="rounded-lg border-solid border border-gray-200 dark:border-gray-600 overflow-clip my-2">
            <div class="bg-[#f6f8fa] dark:bg-gray-900 h-8 border-b border-gray-200 dark:border-gray-600 flex flex-nowrap items-center gap-2 px-2 dark:text-gray-200">
                <span class="font-mono">{props.path.file_name().unwrap_or("")}</span>
            </div>
            <div class="p-2 overflow-x-scroll bg-white">
                <pre class="bg-white">
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
            </div>
        </div>
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
