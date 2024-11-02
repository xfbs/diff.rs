use crate::{cache::*, components::*, data::*, version::VersionId, Route};
use camino::Utf8PathBuf;
use semver::Version;
use std::{collections::BTreeMap, sync::Arc};
use yew::{prelude::*, suspense::*};
use yew_router::prelude::*;

/// Props for which file to show.
#[derive(Properties, PartialEq, Clone)]
pub struct DiffProps {
    pub src_name: String,
    pub dst_name: String,
    pub old: VersionId,
    pub new: VersionId,
    #[prop_or_default]
    pub path: Option<Utf8PathBuf>,
}

/// Show diff of a file change between two crate versions.
#[function_component]
pub fn Diff(props: &DiffProps) -> Html {
    let fallback = html! {
        <>
            <SimpleNavbar />
            <Content>
                <Center>
                    <Loading title={"Loading crate"} status={"Loading crate metadata"} />
                </Center>
            </Content>
        </>
    };
    html! {
        <Suspense {fallback}>
            <CrateFetcher
                src_name={props.src_name.clone()}
                dst_name={props.dst_name.clone()}
                old={props.old.clone()}
                new={props.new.clone()}
                path={props.path.clone()}
            />
        </Suspense>
    }
}

#[function_component]
fn CrateFetcher(props: &DiffProps) -> HtmlResult {
    let info = use_future_with(
        (props.src_name.clone(), props.dst_name.clone()),
        |names| async move {
            (
                CRATE_RESPONSE_CACHE.fetch_cached(&names.0).await,
                CRATE_RESPONSE_CACHE.fetch_cached(&names.1).await,
            )
        },
    )?;

    let errors = match &*info {
        (Ok(src_info), Ok(dst_info)) => {
            return Ok(html! {
                <VersionResolver
                    {src_info}
                    {dst_info}
                    old={props.old.clone()}
                    new={props.new.clone()}
                    path={props.path.clone()}
                />
            })
        }
        (Err(error), Ok(_)) => vec![(&props.src_name, error)],
        (Ok(_), Err(error)) => vec![(&props.dst_name, error)],
        (Err(src_error), Err(dst_error)) => {
            vec![(&props.src_name, src_error), (&props.dst_name, dst_error)]
        }
    };
    let errors = errors
        .iter()
        .map(|(name, error)| format!("{name} with {error}"))
        .collect::<Vec<_>>()
        .join(" and ");
    Ok(html! {
        <>
            <SimpleNavbar />
            <Content>
                <Center>
                    <Error title={"Loading crate"} status={format!("Error: {errors}")} />
                </Center>
            </Content>
        </>
    })
}

#[derive(Properties, PartialEq, Clone)]
struct VersionResolverProps {
    src_info: Arc<CrateResponse>,
    dst_info: Arc<CrateResponse>,
    old: VersionId,
    new: VersionId,
    path: Option<Utf8PathBuf>,
}

#[function_component]
fn VersionResolver(props: &VersionResolverProps) -> Html {
    // find krate version info
    let old = props.src_info.version(props.old.clone());
    let new = props.dst_info.version(props.new.clone());
    let errors = match (old, new) {
        (Some(old), Some(new)) => {
            return html! {
                <SourceFetcher
                    src_info={props.src_info.clone()}
                    dst_info={props.dst_info.clone()}
                    old={old.clone()}
                    new={new.clone()}
                    path={props.path.clone()}
                />
            }
        }
        // get invalid versions from props
        (None, Some(_)) => vec![(&props.src_info, &props.old)],
        (Some(_), None) => vec![(&props.dst_info, &props.new)],
        (None, None) => vec![(&props.src_info, &props.old), (&props.dst_info, &props.new)],
    };
    let errors = errors
        .iter()
        .map(|(info, version)| format!("Error: version {version} of {} not found", info.krate.id))
        .collect::<Vec<_>>()
        .join(" and ");
    html! {
        <>
            <SimpleNavbar />
                <Content>
                    <Center>
                    <Error title={"Resolving version"} status={errors} />
                </Center>
            </Content>
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct SourceFetcherProps {
    src_info: Arc<CrateResponse>,
    dst_info: Arc<CrateResponse>,
    old: VersionInfo,
    new: VersionInfo,
    path: Option<Utf8PathBuf>,
}

#[function_component]
fn SourceFetcher(props: &SourceFetcherProps) -> Html {
    let fallback = html! {
        <>
            <ComplexNavbar
                src_name={props.src_info.krate.id.clone()}
                dst_name={props.dst_info.krate.id.clone()}
                old={props.old.version.clone()}
                new={props.new.version.clone()}
                src_info={props.src_info.clone()}
                dst_info={props.dst_info.clone()}
            />
            <Center>
                <Loading title={"Loading crate"} status={"Loading crate source"} />
            </Center>
        </>
    };
    html! {
        <Suspense {fallback}>
            <SourceFetcherInner
                src_info={props.src_info.clone()}
                dst_info={props.dst_info.clone()}
                old={props.old.clone()}
                new={props.new.clone()}
                path={props.path.clone()}
            />
        </Suspense>
    }
}

#[function_component]
fn SourceFetcherInner(props: &SourceFetcherProps) -> HtmlResult {
    // fetch old version source
    let old = use_future_with(props.old.clone(), |version| async move {
        CRATE_SOURCE_CACHE.fetch_cached(&version).await
    })?;

    // fetch new version source
    let new = use_future_with(props.new.clone(), |version| async move {
        CRATE_SOURCE_CACHE.fetch_cached(&version).await
    })?;

    let navigator = use_navigator().unwrap();
    let (old, new) = match (&*old, &*new) {
        (Ok(old), Ok(new)) => (old, new),
        (Err(error), _) | (_, Err(error)) => {
            return Ok(html! {
                <>
                <ComplexNavbar
                    src_name={props.src_info.krate.id.clone()}
                    dst_name={props.dst_info.krate.id.clone()}
                    old={props.old.version.clone()}
                    new={props.new.version.clone()}
                    src_info={props.src_info.clone()}
                    dst_info={props.dst_info.clone()}
                    onchange={
                        let path = props.path.clone();
                        move |((src_name, old), (dst_name, new)): ((String, Version), (String, Version))| {
                            navigator.push(&Route::File {
                                old_krate: src_name.clone(),
                                new_krate: dst_name.clone(),
                                old_version: old.clone().into(),
                                new_version: new.clone().into(),
                                path: path.clone().unwrap_or_default().into(),
                            });
                        }
                    }
                />
                <Content>
                    <Center>
                        <Error title={"Loading crate"} status={format!("Error: {error}")} />
                    </Center>
                </Content>
                </>
            })
        }
    };

    dbg!(&props.path);
    let path = match &props.path {
        None => {
            return Ok(html! {
                <Redirect<Route> to={Route::File {
                    old_krate: props.src_info.krate.id.clone(),
                    new_krate: props.dst_info.krate.id.clone(),
                    old_version: props.old.version.clone().into(),
                    new_version: props.new.version.clone().into(),
                    path: "Cargo.toml".into(),
                }} />
            })
        }
        Some(path) => path.clone(),
    };

    Ok(html! {
        <div class="">
            <SourceView
                src_info={props.src_info.clone()}
                dst_info={props.dst_info.clone()}
                {old}
                {new}
                {path}
            />
        </div>
    })
}

#[derive(Properties, PartialEq, Clone)]
pub struct SourceViewProps {
    pub src_info: Arc<CrateResponse>,
    pub dst_info: Arc<CrateResponse>,
    pub old: Arc<CrateSource>,
    pub new: Arc<CrateSource>,
    pub path: Utf8PathBuf,
}

#[function_component]
pub fn SourceView(props: &SourceViewProps) -> Html {
    let diff = use_memo((props.old.clone(), props.new.clone()), |(old, new)| {
        VersionDiff::new(old.clone(), new.clone())
    });
    let navigator = use_navigator().unwrap();

    let files = determine_display_files(&FileTreeProps {
        diff: diff.clone(),
        path: props.path.clone(),
    });

    html! {
        <>
            <ComplexNavbar
                src_name={props.src_info.krate.id.clone()}
                dst_name={props.dst_info.krate.id.clone()}
                old={props.old.version.version.clone()}
                new={props.new.version.version.clone()}
                src_info={props.src_info.clone()}
                dst_info={props.dst_info.clone()}
                onchange={
                    let path = props.path.clone();
                    let navigator = navigator;
                    move |((src_name, old), (dst_name, new)): ((String, Version), (String, Version))| {
                        navigator.push(&Route::File {
                            old_krate: src_name.clone(),
                            new_krate: dst_name.clone(),
                            old_version: old.clone().into(),
                            new_version: new.clone().into(),
                            path: path.clone(),
                        });
                    }
                }
            />
            <Content>
                <main class="flex flex-col md:flex-row gap-2 lg:gap-4 p-2">
                    <nav id="files" class="md:w-72 lg:w-84 xl:w-96" aria-label="Files">
                        <FileTree
                            diff={diff.clone()}
                            path={props.path.clone()}
                        />
                    </nav>
                    <div id="diff-view" class="flex-1">
                    { for (*files).iter().map(|path| html!{<LazyDiffView diff={diff.clone()} path={path.clone()} /> }) }
                    </div>
                </main>
            </Content>
        </>
    }
}

fn determine_display_files(props: &FileTreeProps) -> Vec<Utf8PathBuf> {
    if props.diff.files.contains_key(&props.path) {
        // found a file e \o/
        return vec![props.path.clone()];
    }

    let path = props.path.clone();
    let mut navigator = &props.diff.tree.item;

    for path_components in path.components() {
        if let Item::Dir(dir) = navigator {
            if let Some(deeper) = dir.get(&path_components.to_string()) {
                if deeper.changes != Changes::default() {
                    navigator = &deeper.item;
                    continue;
                }
            }
        }

        return vec![path.clone()];
    }

    let files_in_path: Vec<Utf8PathBuf> = if let Item::Dir(dir) = navigator {
        rel_filepaths_in_tree(dir, true)
            .into_iter()
            .map(|p| {
                let mut dirn_name = props.path.clone();
                dirn_name.push(&p);
                dirn_name
            })
            .collect()
    } else {
        unreachable!()
    };

    if files_in_path.is_empty() {
        vec![props.path.clone()]
    } else {
        files_in_path
    }
}

fn rel_filepaths_in_tree(
    dir: &BTreeMap<String, std::rc::Rc<Entry>>,
    no_unchanged: bool,
) -> Vec<Utf8PathBuf> {
    let mut paths: Vec<Utf8PathBuf> = Vec::new();
    for (name, entry) in dir {
        if entry.changes == Changes::default() && no_unchanged {
            continue;
        }

        match entry.item {
            Item::Dir(ref dir) => {
                let mut sub_tree_files = rel_filepaths_in_tree(dir, no_unchanged)
                    .into_iter()
                    .map(|files_relative_path| {
                        let mut cur_dir = Utf8PathBuf::from(name);
                        cur_dir.push(&files_relative_path);
                        cur_dir
                    })
                    .collect();
                paths.append(&mut sub_tree_files);
            }
            Item::File => {
                paths.push(name.into());
            }
        }
    }

    paths
}
