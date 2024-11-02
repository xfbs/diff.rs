use crate::{
    data::{ChunkInfo, FileDiff, VersionDiff},
    syntax::{highlight_changes, infer_syntax_for_file, syntect_style_to_css},
};
use bytes::Bytes;
use camino::Utf8PathBuf;
use log::*;
use similar::ChangeTag;
use std::rc::Rc;
use syntect::highlighting::Style;
use yew::prelude::*;
use yew_hooks::use_visible;

/// Contains information about contiguous changes
#[derive(PartialEq, Clone)]
struct DiffGroupInfo {
    /// The actual changes
    group: Vec<(ChangeTag, Vec<(Style, bytes::Bytes)>)>,
    /// What range of lines the group covers (used as a Yew list key)
    range: ChunkInfo,
    /// Whether the group contains an actual diff (and therefore shows some context)
    in_context: bool,
}

#[derive(Properties, PartialEq, Clone)]
pub struct DiffViewProps {
    pub path: Utf8PathBuf,
    pub diff: Rc<VersionDiff>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum DiffStyle {
    #[default]
    Unified,
    Split,
}

#[function_component]
fn FileIcon() -> Html {
    // from https://www.svgrepo.com/svg/491619/doc
    html! {
        <svg class="fill-gray-500 w-4" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M6 1C4.34314 1 3 2.34315 3 4V20C3 21.6569 4.34315 23 6 23H19C20.6569 23 22 21.6569 22 20V10C22 9.73478 21.8946 9.48043 21.7071 9.29289L13.7071 1.29292C13.6114 1.19722 13.4983 1.1229 13.3753 1.07308C13.2572 1.02527 13.1299 1 13 1H6ZM12 3H6C5.44771 3 5 3.44771 5 4V20C5 20.5523 5.44772 21 6 21H19C19.5523 21 20 20.5523 20 20V11H13C12.4477 11 12 10.5523 12 10V3ZM18.5858 9.00003L14 4.41424V9.00003H18.5858Z" />
        </svg>
    }
}

#[function_component]
pub fn DiffView(props: &DiffViewProps) -> Html {
    let empty = FileDiff::default();
    let file_diff = props.diff.files.get(&props.path).unwrap_or(&empty);
    let summary = props.diff.summary.get(&props.path).unwrap_or(&(0, 0));
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
        if next_range.start() != 0 {
            stack.push(DiffGroupInfo {
                group: changes.by_ref().take(next_range.start() - cursor).collect(),
                range: ChunkInfo {
                    range: cursor..next_range.start(),
                    left_start: (next_range.left_start + cursor).saturating_sub(next_range.start()),
                    right_start: (next_range.right_start + cursor)
                        .saturating_sub(next_range.start()),
                },
                in_context: false,
            });
        }
        // in context lines
        stack.push(DiffGroupInfo {
            group: changes
                .by_ref()
                .take(next_range.end() - next_range.start())
                .collect(),
            range: next_range.clone(),
            in_context: true,
        });
        cursor = next_range.end();
    }
    if changes.len() > 0 {
        // Trailing unchanged lines at the end of a file
        stack.push(DiffGroupInfo {
            group: changes.by_ref().collect(),
            range: ChunkInfo {
                range: cursor..file_diff.changes.len(),
                left_start: (cursor).saturating_sub(file_diff.summary.added as usize),
                right_start: (cursor).saturating_sub(file_diff.summary.removed as usize),
            },

            // When comparing a version of the crate to itself, this group will
            // always contain the full text of the file. Don't collapse it.
            in_context: is_identical_version,
        });
    }

    html! {
        <div class="diff-view">
            <div class="header">
                <FileIcon />
                <span class="filename">{props.path.file_name().unwrap_or("")}</span>
            </div>
            <div class="content">
                {
                    if summary == &(0,0) {
                        html! {<FileDisplayView {stack} />}
                    } else {
                        html! {<UnifiedDiffView {stack} />}
                    }
                }
            </div>
        </div>
    }
}

#[function_component]
pub fn LazyDiffView(props: &DiffViewProps) -> Html {
    let node = use_node_ref();
    let visible = use_visible(node.clone(), true);

    let len_estimation = if visible {
        // no need to estimate
        None
    } else {
        // just to clarify: thisis obviously wrong for files w/o changes (but atm those files are visible anyway)
        let empty = FileDiff::default();
        let file_diff = props.diff.files.get(&props.path).unwrap_or(&empty);
        Some(
            file_diff
                .context_ranges
                .iter()
                .map(|chunk| chunk.len())
                .sum::<usize>(),
        )
    };

    html! {
        <div ref={node}>
            if visible {
                    <DiffView diff={props.diff.clone()} path={props.path.clone()} />
            } else {
               <div style={format!("height: {}px; width: 100%;", len_estimation.unwrap_or(0)*24)} />
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct AnyDiffViewProps {
    stack: Vec<DiffGroupInfo>,
}

#[function_component]
pub fn UnifiedDiffView(props: &AnyDiffViewProps) -> Html {
    let mut overall_index = 0;
    html! {
        <div class="overflow-x-scroll bg-white">
            <div class="unified">
            {
                props.stack.iter()
                    .map(|DiffGroupInfo {group, range, in_context}| {
                        let res = html!{
                            <DiffLineGroup
                                key={format!("{:?}", range.range)}
                                group={group.clone()}
                                {in_context}
                                group_start_index={(overall_index, range.left_start, range.right_start)}
                            />
                        };
                        overall_index += group.len();
                        res
                    })
                    .collect::<Html>()
            }
            </div>
        </div>
    }
}

#[function_component]
pub fn FileDisplayView(props: &AnyDiffViewProps) -> Html {
    let mut overall_index = 0;
    html! {
        <div class="overflow-x-scroll bg-white">
            <div class="unified">
            {
                props.stack.iter()
                    .map(|DiffGroupInfo {group, range, in_context: _}| {
                        let res = html!{
                            <FileView
                                key={format!("{:?}", range)}
                                group={group.iter().map(|(_, line)| line.clone()).collect::<Vec<_>>()}
                                group_start_index={overall_index}
                            />
                        };
                        overall_index += group.len();
                        res
                    })
                    .collect::<Html>()
            }
            </div>
        </div>
    }
}

#[function_component]
pub fn SplitDiffView(props: &AnyDiffViewProps) -> Html {
    let mut overall_index = 0;
    html! {
        <div class="p-2 overflow-x-scroll bg-white">
            <pre class="bg-white">
            {
                props.stack.iter()
                    .map(|DiffGroupInfo {group, range, in_context}| {
                        let res = html!{
                            <DiffLineGroup
                                key={format!("{:?}", range)}
                                group={group.clone()}
                                {in_context}
                                group_start_index={(overall_index, range.left_start, range.right_start)}
                            />
                        };
                        overall_index += group.len();
                        res
                    })
                    .collect::<Html>()
            }
            </pre>
        </div>
    }
}

#[function_component]
fn ExpandIcon() -> Html {
    html! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="currentColor" height="1em" width="1em" class="inline">
            <path d="m8.177.677 2.896 2.896a.25.25 0 0 1-.177.427H8.75v1.25a.75.75 0 0 1-1.5 0V4H5.104a.25.25 0 0 1-.177-.427L7.823.677a.25.25 0 0 1 .354 0ZM7.25 10.75a.75.75 0 0 1 1.5 0V12h2.146a.25.25 0 0 1 .177.427l-2.896 2.896a.25.25 0 0 1-.354 0l-2.896-2.896A.25.25 0 0 1 5.104 12H7.25v-1.25Zm-5-2a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5ZM6 8a.75.75 0 0 1-.75.75h-.5a.75.75 0 0 1 0-1.5h.5A.75.75 0 0 1 6 8Zm2.25.75a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5ZM12 8a.75.75 0 0 1-.75.75h-.5a.75.75 0 0 1 0-1.5h.5A.75.75 0 0 1 12 8Zm2.25.75a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5Z"></path>
        </svg>
    }
}

#[derive(Properties, PartialEq)]
pub struct DiffLineGroupProps {
    group: Vec<(ChangeTag, Vec<(Style, bytes::Bytes)>)>,
    in_context: bool,
    group_start_index: (usize, usize, usize),
}

#[derive(Properties, PartialEq)]
pub struct DisplayGroupProps {
    group: Vec<Vec<(Style, bytes::Bytes)>>,
    group_start_index: usize,
}

#[function_component]
pub fn DiffLineGroup(props: &DiffLineGroupProps) -> Html {
    let folded = use_state(|| !props.in_context);
    let onclick = {
        let folded = folded.clone();
        Callback::from(move |_| folded.set(!*folded))
    };

    // go from 0-indexed to 1-indexed
    let start_index = (
        props.group_start_index.0 + 1,
        props.group_start_index.1 + 1,
        props.group_start_index.2 + 1,
    );

    // use the fact that folded sections never contain changes
    let end_index = (
        start_index.0 + props.group.len() - 1,
        start_index.1 + props.group.len() - 1,
        start_index.2 + props.group.len() - 1,
    );

    if *folded {
        html! {
            <div class="expand">
                <button class={classes!("button")} onclick={onclick.clone()}>
                    <ExpandIcon />
                </button>
                <button class={classes!("info")} {onclick}>
                    {
                    if start_index.1 == start_index.2 {
                        format!("Show lines {:?} to {:?}", start_index.1, end_index.1)
                    } else {
                        format!("Show lines {:?} to {:?}", (start_index.1,start_index.2), (end_index.1,end_index.2))
                    }
                }
                </button>
            </div>
        }
    } else {
        let (mut left_idx, mut right_idx) = (start_index.1, start_index.2);
        html! {
            <>
            {
                props.group.iter().map(|(tag, change)| {
                    let (sign, class, left, right) = match tag {
                        ChangeTag::Delete => ("-", "deletion", Some(left_idx), None),
                        ChangeTag::Insert => ("+", "insertion", None, Some(right_idx)),
                        ChangeTag::Equal => (" ", "unchanged", Some(left_idx), Some(right_idx)),
                    };
                    (left_idx, right_idx) = match tag {
                        ChangeTag::Delete => (left_idx + 1, right_idx),
                        ChangeTag::Insert => (left_idx, right_idx + 1),
                        ChangeTag::Equal => (left_idx + 1, right_idx + 1),
                    };

                    html! {
                        <div class={classes!("line", class)}>
                            <a id={left.map(|i| format!("L{i}"))} class="line-number">
                                if let Some(index) = left {
                                    {index}
                                }
                            </a>
                            <a id={right.map(|i| format!("R{i}"))} class="line-number">
                                if let Some(index) = right {
                                    {index}
                                }
                            </a>
                            <div class="change-icon">
                                {
                                    format!("{sign}")
                                }
                            </div>
                            <div class="code-line">
                                <CodeLine stack={change.clone()} />
                            </div>
                        </div>
                    }
                }).collect::<Html>()
            }
            </>
        }
    }
}

#[function_component]
pub fn FileView(props: &DisplayGroupProps) -> Html {
    props
        .group
        .iter()
        .enumerate()
        .map(|(index, change)| {
            html! {
                <div class={classes!("line", "unchanged")}>
                    <div class={classes!("line-number", "file-view")}>
                        {
                            format!("{}", index+1+ props.group_start_index)
                        }
                    </div>
                    <div class="code-line">
                        <CodeLine stack={change.clone()} />
                    </div>
                </div>
            }
        })
        .collect::<Html>()
}

#[derive(Properties, PartialEq)]
pub struct CodeLineProps {
    stack: Vec<(Style, Bytes)>,
}

#[function_component]
pub fn CodeLine(props: &CodeLineProps) -> Html {
    props
        .stack
        .iter()
        .map(|(style, text)| {
            let style = syntect_style_to_css(style);
            let contents = String::from_utf8_lossy(&text[..]);
            html! {
                <span style={style}>{contents}</span>
            }
        })
        .collect::<Html>()
}
