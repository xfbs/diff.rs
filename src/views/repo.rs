use crate::{
    cache::*,
    components::*,
    data::{CrateResponse, CrateSource, RepositoryInfo, VersionDiff, VersionInfo},
    version::VersionId,
};
use camino::Utf8PathBuf;
use log::*;
use std::{rc::Rc, sync::Arc};
use yew::{prelude::*, suspense::*};

#[derive(Properties, PartialEq)]
pub struct RepoFileViewProps {
    pub krate: String,
    pub version: VersionId,
    pub path: Utf8PathBuf,
}

#[function_component]
pub fn RepoFileView(props: &RepoFileViewProps) -> Html {
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
                krate={props.krate.clone()}
                version={props.version.clone()}
                path={props.path.clone()}
            />
        </Suspense>
    }
}

#[function_component]
fn CrateFetcher(props: &RepoFileViewProps) -> HtmlResult {
    let info = use_future_with(props.krate.clone(), |krate| async move {
        CRATE_RESPONSE_CACHE.fetch_cached(&krate).await
    })?;

    let info = match &*info {
        Ok(info) => info.clone(),
        Err(error) => {
            return Ok(html! {
                <>
                    <SimpleNavbar />
                    <Content>
                        <Center>
                            <Error title={"Loading crate"} status={format!("Error: {error}")} />
                        </Center>
                    </Content>
                </>
            });
        }
    };

    Ok(html! {
        <VersionResolver
            info={info.clone()}
            version={props.version.clone()}
            path={props.path.clone()}
        />
    })
}

#[derive(Properties, PartialEq)]
pub struct VersionResolverProps {
    info: Arc<CrateResponse>,
    version: VersionId,
    path: Utf8PathBuf,
}

#[function_component]
fn VersionResolver(props: &VersionResolverProps) -> Html {
    let Some(version) = &props.info.version(props.version.clone()) else {
        return html! {
            {"version not found"}
        };
    };

    html! {
        <CrateSourceFetcher
            info={props.info.clone()}
            version={(*version).clone()}
            path={props.path.clone()}
        />
    }
}

#[derive(Properties, PartialEq)]
pub struct CrateSourceFetcherProps {
    info: Arc<CrateResponse>,
    version: VersionInfo,
    path: Utf8PathBuf,
}

#[function_component]
fn CrateSourceFetcher(props: &CrateSourceFetcherProps) -> Html {
    html! {
        <Suspense fallback={html!{{"Loading"}}}>
            <CrateSourceFetcherInner info={props.info.clone()} version={props.version.clone()} path={props.path.clone()} />
        </Suspense>
    }
}

#[function_component]
fn CrateSourceFetcherInner(props: &CrateSourceFetcherProps) -> HtmlResult {
    let source = use_future_with(props.version.clone(), |version| async move {
        CRATE_SOURCE_CACHE.fetch_cached(&version).await
    })?;

    let source = match &*source {
        Ok(source) => source.clone(),
        Err(error) => {
            return Ok(html! {
                {"Error fetching source"}
            });
        }
    };

    Ok(html! {
        <RepoSourceFetcher
            info={props.info.clone()}
            {source}
            path={props.path.clone()}
        />
    })
}

#[derive(Properties, PartialEq)]
pub struct RepoSourceFetcherProps {
    info: Arc<CrateResponse>,
    source: Arc<CrateSource>,
    path: Utf8PathBuf,
}

#[function_component]
fn RepoSourceFetcher(props: &RepoSourceFetcherProps) -> Html {
    let Some(repository) = props.info.krate.repository.clone() else {
        return html! {
            {"No repository set in crate metadata"}
        };
    };

    let vcs_info = match props.source.cargo_vcs_info() {
        Ok(info) => info,
        Err(error) => {
            return html! {
                <>
                {"error: "}{error.to_string()}
                </>
            };
        }
    };

    let info = RepositoryInfo {
        repository,
        vcs_info,
    };

    let url = info.url();

    let fallback = html! {
        {"Loading repository archive"}
    };
    html! {
        <Suspense {fallback}>
            <RepoSourceFetcherInner
                info={props.info.clone()}
                repository={info}
                source={props.source.clone()}
                path={props.path.clone()}
            />
        </Suspense>
    }
}

#[derive(Properties, PartialEq)]
pub struct RepoSourceFetcherInnerProps {
    info: Arc<CrateResponse>,
    source: Arc<CrateSource>,
    repository: RepositoryInfo,
    path: Utf8PathBuf,
}

#[function_component]
fn RepoSourceFetcherInner(props: &RepoSourceFetcherInnerProps) -> HtmlResult {
    let source = use_future_with(props.repository.clone(), |repository| async move {
        repository.fetch().await.map(Arc::new)
    })?;

    let source = match &*source {
        Ok(source) => source.clone(),
        Err(error) => {
            return Ok(html! {
                {format!("Error repo: {error}")}
            })
        }
    };

    let diff = VersionDiff::new(source.clone(), props.source.clone());
    let diff = Rc::new(diff);

    Ok(html! {
            <Content>
                <main class="flex flex-col md:flex-row gap-2 lg:gap-4 p-2">
                    <nav id="files" class="md:w-72 lg:w-84 xl:w-96" aria-label="Files">
                        <FileTree
                            diff={diff.clone()}
                            path={props.path.clone()}
                        />
                    </nav>
                    <div id="diff-view" class="flex-1">
                        <DiffView {diff} path={props.path.clone()} />
                    </div>
                </main>
            </Content>
    })
}
