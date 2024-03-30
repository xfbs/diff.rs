use super::*;
use crate::version::VersionId;
use semver::Version;

#[derive(Properties, PartialEq, Clone)]
pub struct DiffProps {
    pub name: String,
    pub old: VersionId,
    pub new: VersionId,
    #[prop_or_default]
    pub path: Option<String>,
}

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
                name={props.name.clone()}
                old={props.old.clone()}
                new={props.new.clone()}
                path={props.path.clone()}
            />
        </Suspense>
    }
}

#[function_component]
pub fn CrateFetcher(props: &DiffProps) -> HtmlResult {
    let info = use_future_with(props.name.clone(), |name| async move {
        CRATE_RESPONSE_CACHE.fetch_cached(&name).await
    })?;

    match &*info {
        Ok(info) => Ok(html! {
            <VersionResolver {info} old={props.old.clone()} new={props.new.clone()} path={props.path.clone()} />
        }),
        Err(error) => Ok(html! {
            <>
                <SimpleNavbar />
                <Content>
                    <Center>
                        <Error title={"Loading crate"} status={format!("Error: {error}")} />
                    </Center>
                </Content>
            </>
        }),
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct VersionResolverProps {
    pub info: Arc<CrateResponse>,
    pub old: VersionId,
    pub new: VersionId,
    pub path: Option<String>,
}

#[function_component]
pub fn VersionResolver(props: &VersionResolverProps) -> Html {
    // find krate version info
    let old = props.info.version(props.old.clone());
    let new = props.info.version(props.new.clone());

    match (old, new) {
        (Some(old), Some(new)) => html! {
            <SourceFetcher info={props.info.clone()} old={old.clone()} new={new.clone()} path={props.path.clone()} />
        },
        (None, _) => html! {
            <>
            <SimpleNavbar />
            <Content>
            <Center>
            <Error title={"Resolving version"} status={format!("Error: version {old:?} not found")} />
            </Center>
            </Content>
            </>
        },
        (_, None) => html! {
            <>
            <SimpleNavbar />
            <Content>
            <Center>
            <Error title={"Resolving version"} status={format!("Error: version {new:?} not found")} />
            </Center>
            </Content>
            </>
        },
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct SourceFetcherProps {
    pub info: Arc<CrateResponse>,
    pub old: VersionInfo,
    pub new: VersionInfo,
    pub path: Option<String>,
}

#[function_component]
pub fn SourceFetcher(props: &SourceFetcherProps) -> Html {
    let fallback = html! {
        <>
        <ComplexNavbar
            name={props.info.krate.id.clone()}
            old={props.old.num.clone()}
            new={props.new.num.clone()}
            info={props.info.clone()}
        />
        <Center>
            <Loading title={"Loading crate"} status={"Loading crate source"} />
        </Center>
        </>
    };
    html! {
        <Suspense {fallback}>
            <SourceFetcherInner
                info={props.info.clone()}
                old={props.old.clone()}
                new={props.new.clone()}
                path={props.path.clone()}
            />
        </Suspense>
    }
}

#[function_component]
pub fn SourceFetcherInner(props: &SourceFetcherProps) -> HtmlResult {
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
                    name={props.info.krate.id.clone()}
                    old={props.old.num.clone()}
                    new={props.new.num.clone()}
                    info={props.info.clone()}
                    onchange={
                        let name = props.info.krate.id.clone();
                        let path = props.path.clone();
                        move |(old, new): (Version, Version)| {
                            navigator.push(&Route::File {
                                name: name.clone(),
                                old: old.clone().into(),
                                new: new.clone().into(),
                                path: path.clone().unwrap_or_default(),
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

    let path = match &props.path {
        None => {
            return Ok(html! {
                <Redirect<Route> to={Route::File {
                    name: props.info.krate.id.clone(),
                    old: props.old.num.clone().into(),
                    new: props.new.num.clone().into(),
                    path: "Cargo.toml".into(),
                }} />
            })
        }
        Some(path) => path.clone(),
    };

    Ok(html! {
        <SourceView
            info={props.info.clone()}
            {old}
            {new}
            {path}
        />
    })
}
