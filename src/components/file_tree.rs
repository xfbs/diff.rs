use crate::{
    data::{Changes, Entry, Item, VersionDiff},
    Link, Route, VersionId,
};
use camino::Utf8PathBuf;
use std::rc::Rc;
use yew::prelude::*;

#[derive(PartialEq, Clone, Debug)]
struct Context {
    old_krate: String,
    old_version: VersionId,
    new_krate: String,
    new_version: VersionId,
}

impl Context {
    fn file_route(&self, path: Utf8PathBuf) -> Route {
        Route::File {
            old_krate: self.old_krate.clone(),
            old_version: self.old_version.clone(),
            new_krate: self.new_krate.clone(),
            new_version: self.new_version.clone(),
            path,
        }
        .simplify()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum ChangeFilter {
    #[default]
    All,
    Changed,
}

impl ChangeFilter {
    fn is_all(&self) -> bool {
        matches!(self, Self::All)
    }

    fn is_changed(&self) -> bool {
        matches!(self, Self::Changed)
    }

    fn matches(&self, changes: Changes) -> bool {
        match self {
            Self::All => true,
            Self::Changed => changes != Changes::default(),
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct FileTreeProps {
    pub diff: Rc<VersionDiff>,
    pub path: Utf8PathBuf,
}

#[derive(Properties, PartialEq, Clone)]
struct SubTreeProps {
    pub context: Rc<Context>,
    pub entry: Rc<Entry>,
    #[prop_or_default]
    pub active: Rc<Utf8PathBuf>,
    #[prop_or_default]
    pub prefix: Rc<Utf8PathBuf>,
    #[prop_or_default]
    pub change_filter: ChangeFilter,
}

#[derive(Properties, PartialEq, Clone)]
struct FileEntryProps {
    pub context: Rc<Context>,
    pub entry: Rc<Entry>,
    #[prop_or_default]
    pub active: Rc<Utf8PathBuf>,
    #[prop_or_default]
    pub prefix: Rc<Utf8PathBuf>,
    #[prop_or_default]
    pub change_filter: ChangeFilter,
}

#[function_component]
fn FolderIcon() -> Html {
    // from https://www.svgrepo.com/svg/491619/doc
    html! {
        <svg class="fill-blue-300 dark:fill-gray-600" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <path d="M4 2C3.20435 2 2.44129 2.31607 1.87868 2.87868C1.31607 3.44129 1 4.20435 1 5V19C1 19.7957 1.31607 20.5587 1.87868 21.1213C2.44129 21.6839 3.20435 22 4 22H20C20.7957 22 21.5587 21.6839 22.1213 21.1213C22.6839 20.5587 23 19.7957 23 19V8C23 7.20435 22.6839 6.44129 22.1213 5.87868C21.5587 5.31607 20.7957 5 20 5H11.5352L10.1289 2.8906C9.75799 2.3342 9.13352 2 8.46482 2H4Z" />
        </svg>
    }
}

#[function_component]
fn FileIcon() -> Html {
    // from https://www.svgrepo.com/svg/491619/doc
    html! {
        <svg class="fill-gray-500" viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M6 1C4.34314 1 3 2.34315 3 4V20C3 21.6569 4.34315 23 6 23H19C20.6569 23 22 21.6569 22 20V10C22 9.73478 21.8946 9.48043 21.7071 9.29289L13.7071 1.29292C13.6114 1.19722 13.4983 1.1229 13.3753 1.07308C13.2572 1.02527 13.1299 1 13 1H6ZM12 3H6C5.44771 3 5 3.44771 5 4V20C5 20.5523 5.44772 21 6 21H19C19.5523 21 20 20.5523 20 20V11H13C12.4477 11 12 10.5523 12 10V3ZM18.5858 9.00003L14 4.41424V9.00003H18.5858Z" />
        </svg>
    }
}

#[function_component]
fn ExpandIcon() -> Html {
    html! {
        <svg viewBox="0 0 24 24" fill="currentColor" xmlns="http://www.w3.org/2000/svg">
            <path fill-rule="evenodd" clip-rule="evenodd" d="M8.79289 6.29289C9.18342 5.90237 9.81658 5.90237 10.2071 6.29289L15.2071 11.2929C15.5976 11.6834 15.5976 12.3166 15.2071 12.7071L10.2071 17.7071C9.81658 18.0976 9.18342 18.0976 8.79289 17.7071C8.40237 17.3166 8.40237 16.6834 8.79289 16.2929L13.0858 12L8.79289 7.70711C8.40237 7.31658 8.40237 6.68342 8.79289 6.29289Z" />
        </svg>
    }
}

#[function_component]
fn FileEntry(props: &FileEntryProps) -> Html {
    let path = {
        let mut path = (*props.prefix).clone();
        path.push(&props.entry.name);
        path
    };

    let expanded = use_state(|| false);
    let current = path == *props.active;

    let route = props.context.file_route(path.clone());

    let expand = |event: MouseEvent| {
        event.prevent_default();
    };

    html! {
        <>
        <Link to={route} classes={classes!("file-entry", current.then_some("active"))}>
            <button class={classes!("toggle", (*expanded).then_some("active"))} onclick={expand}>
                if props.entry.item.is_dir() {
                    <ExpandIcon />
                }
            </button>
            <div class="icon">
                if props.entry.item.is_dir() {
                    <FolderIcon />
                } else {
                    <FileIcon />
                }
            </div>
            <div class="name">
                {&props.entry.name}
            </div>
            <div class="tags">
                if props.entry.changes.added > 0 {
                    <span class="tag added">{"+"}{props.entry.changes.added}</span>
                }
                if props.entry.changes.removed > 0 {
                    <span class="tag removed">{"-"}{props.entry.changes.removed}</span>
                }
            </div>
        </Link>
        if props.entry.item.is_dir() {
            <SubTree
                entry={props.entry.clone()}
                context={props.context.clone()}
                prefix={props.prefix.clone()}
                active={props.active.clone()}
                change_filter={props.change_filter}
            />
        }
        </>
    }
}

#[function_component]
fn SubTree(props: &SubTreeProps) -> Html {
    debug_assert!(props.entry.item.is_dir());

    // build new prefix
    let mut prefix = (*props.prefix).clone();
    prefix.push(&props.entry.name);
    let prefix = Rc::new(prefix);

    let entries = match &props.entry.item {
        Item::File => unreachable!(),
        Item::Dir(entries) => entries,
    };

    html! {
        <div class="file-subtree">
        {
            entries
                .iter()
                .filter(|(_, entry)| props.change_filter.matches(entry.changes))
                .map(|(key, entry)| html! {
                    <FileEntry
                        key={key.to_string()}
                        context={props.context.clone()}
                        entry={entry.clone()}
                        prefix={prefix.clone()}
                        active={props.active.clone()}
                        change_filter={props.change_filter}
                    />
                })
                .collect::<Html>()
        }
        </div>
    }
}

#[function_component]
pub fn FileTree(props: &FileTreeProps) -> Html {
    let entries = match props.diff.tree.item.clone() {
        Item::File => Default::default(),
        Item::Dir(entries) => entries,
    };

    let change_filter = use_state(|| ChangeFilter::All);
    let change_filter_set = |filter: ChangeFilter| {
        let change_filter = change_filter.clone();
        move |event: MouseEvent| {
            change_filter.set(filter);
            event.prevent_default();
        }
    };

    let prefix = Rc::new(Utf8PathBuf::default());
    let active = Rc::new(props.path.clone());

    let context = Rc::new(Context {
        old_krate: props.diff.left.version.krate.clone(),
        old_version: props.diff.left.version.num.clone().into(),
        new_krate: props.diff.right.version.krate.clone(),
        new_version: props.diff.right.version.num.clone().into(),
    });

    html! {
        <div class="file-tree">
            <div class="header">
                <span></span>
                <span class="grow"></span>
                <div class="button-group" role="group">
                    <button
                        type="button"
                        class={classes!("first", change_filter.is_all().then(|| "active"))}
                        onclick={change_filter_set(ChangeFilter::All)}>
                        {"all"}
                    </button>
                    <button
                        type="button"
                        class={classes!("last", change_filter.is_changed().then(|| "active"))}
                        onclick={change_filter_set(ChangeFilter::Changed)}>
                        {"changed"}
                    </button>
                </div>
            </div>
        {
            entries
                .into_iter()
                .filter(|(_, entry)| change_filter.matches(entry.changes))
                .map(|(key, entry)| html! {
                    <FileEntry
                        {key}
                        {entry}
                        prefix={prefix.clone()}
                        active={active.clone()}
                        context={context.clone()}
                        change_filter={*change_filter}
                    />
                })
                .collect::<Html>()
        }
        </div>
    }
}
