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

#[derive(Debug, PartialEq, Copy, Clone)]
enum HunkVisibilityAction {
    ShowFull,
    HideFull,
    StartMore,
    StartFewer,
    EndMore,
    EndFewer,
}

fn compute_hunk_visibility_change(
    change: HunkVisibilityAction,
    range_to_hide: Option<&std::ops::Range<usize>>,
    full_range: &std::ops::Range<usize>,
) -> Option<std::ops::Range<usize>> {
    // Simple cases first
    match change {
        HunkVisibilityAction::ShowFull => return None,
        HunkVisibilityAction::HideFull => return Some(full_range.clone()),
        _ => {}
    };

    let range = range_to_hide?;

    match change {
        HunkVisibilityAction::StartMore => {
            let new_range = (range.start + 20).min(full_range.end)..range.end;
            if new_range.start >= new_range.end {
                None
            } else {
                Some(new_range)
            }
        }
        HunkVisibilityAction::StartFewer => {
            let new_range = range.start.saturating_sub(20).max(full_range.start)..range.end;
            if new_range.start >= new_range.end {
                None
            } else {
                Some(new_range)
            }
        }
        HunkVisibilityAction::EndMore => {
            let new_range = range.start..range.end.saturating_sub(20).max(full_range.start);
            if new_range.start >= new_range.end {
                None
            } else {
                Some(new_range)
            }
        }
        HunkVisibilityAction::EndFewer => {
            let new_range = range.start..(range.end + 20).min(full_range.end);
            if new_range.start >= new_range.end {
                None
            } else {
                Some(new_range)
            }
        }
        _ => unreachable!(),
    }
}

fn hunk_visibility_callback<T>(
    change: HunkVisibilityAction,
    range_to_hide: UseStateHandle<Option<std::ops::Range<usize>>>,
    full_range: &std::ops::Range<usize>,
) -> Callback<T> {
    let new_range = compute_hunk_visibility_change(change, range_to_hide.as_ref(), full_range);
    Callback::from(move |_| range_to_hide.set(new_range.clone()))
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
    let full_range = &props.range;
    let range_to_hide = use_state(|| {
        if props.in_context {
            None
        } else {
            Some(full_range.clone())
        }
    });

    let padding = props.padding;

    let range_class = if props.in_context {
        "range in-context"
    } else {
        "range out-of-context"
    };

    html! {
        <>
        if range_to_hide.is_none() {
            <FoldingContext
                range_to_hide={range_to_hide.clone()}
                full_range={full_range.clone()}
                in_context={props.in_context}
            />
        }
        <div class={range_class}>
            {
                props.group.iter().enumerate().filter_map(|(index, (tag, change))| {
                    let current_line = props.range.start + index;
                    if let Some(range) =  range_to_hide.as_ref() {
                        let in_range = !(
                            current_line >= range.start
                            &&
                            current_line <= range.end
                        );
                        if !in_range {
                            if current_line == range.start {
                                return Some(
                                    html! {
                                        <FoldingContext
                                            range_to_hide={range_to_hide.clone()}
                                            full_range={full_range.clone()}
                                            in_context={props.in_context}
                                        />
                                    }
                                )
                            }
                            return None
                        }
                    };

                    let overall_index = props.range.start + index;
                    let (sign, color) = match tag {
                        ChangeTag::Delete => ("-", "red"),
                        ChangeTag::Insert => ("+", "green"),
                        ChangeTag::Equal => (" ", "default"),
                    };
                    let contents = String::from_utf8_lossy(&change[..]);
                    log::debug!("{:?} {}", range_to_hide, overall_index);
                    Some(html! {
                        <span style={format!("color: {color};")}>
                            {
                                format!(
                                    "{overall_index:>padding$} {sign} {}",
                                    contents
                                )
                            }
                        </span>
                    })
                }).collect::<Html>()
        }
        </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct FoldingContextProps {
    range_to_hide: UseStateHandle<Option<std::ops::Range<usize>>>,
    full_range: std::ops::Range<usize>,
    in_context: bool,
}

#[function_component]
pub fn FoldingContext(props: &FoldingContextProps) -> Html {
    let action =
        |action| hunk_visibility_callback(action, props.range_to_hide.clone(), &props.full_range);

    let full_range = &props.full_range;
    let in_context = props.in_context;

    let start_more;
    let start_fewer;
    let show_full;
    let hide_full;
    let end_more;
    let end_fewer;

    match props.range_to_hide.as_ref() {
        None => {
            start_more = false;
            start_fewer = false;
            show_full = false;
            hide_full = true;
            end_more = false;
            end_fewer = false;
        }
        Some(range) => {
            start_more = !in_context && range.end > range.start;
            start_fewer = !in_context && range.start > full_range.start;
            show_full = true;
            hide_full = range.end != full_range.end || range.start != full_range.start;
            end_more = !in_context && range.end > range.start;
            end_fewer = !in_context && range.end < full_range.end;
        }
    };

    html! {
        <div class="folding-sticky">
            if start_fewer {
                <button class="start-fewer" onclick={action(HunkVisibilityAction::StartFewer)}>
                    {format!("Show fewer start lines")}
                </button>
            }
            if start_more {
                <button class="start-more" onclick={action(HunkVisibilityAction::StartMore)}>
                    {format!("Show more start lines")}
                </button>
            }
            if show_full {
                <button class="show-full" onclick={action(HunkVisibilityAction::ShowFull)}>
                    {format!("Show lines {} to {}", full_range.start, full_range.end)}
                </button>
            }
            if hide_full {
                <button class="hide-full" onclick={action(HunkVisibilityAction::HideFull)}>
                    {format!("Fold lines {} to {}", full_range.start, full_range.end)}
                </button>
            }
            if end_more {
                <button class="end-more" onclick={action(HunkVisibilityAction::EndMore)}>
                    {format!("Show more end lines")}
                </button>
            }
            if end_fewer {
                <button class="end-fewer" onclick={action(HunkVisibilityAction::EndFewer)}>
                    {format!("Show fewer end lines")}
                </button>
            }
        </div>
    }
}
