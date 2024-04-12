use super::*;
use crate::version::VersionId;
use semver::Version;

#[derive(Properties, PartialEq, Clone)]
pub struct DiffProps {
    pub src_name: String,
    pub dst_name: String,
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
pub fn CrateFetcher(props: &DiffProps) -> HtmlResult {
    let info = use_future_with(
        (props.src_name.clone(), props.dst_name.clone()),
        |names| async move {
            (
                CRATE_RESPONSE_CACHE.fetch_cached(&names.0).await,
                CRATE_RESPONSE_CACHE.fetch_cached(&names.1).await,
            )
        },
    )?;

    match &*info {
        (Ok(src_info), Ok(dst_info)) => Ok(html! {
            <VersionResolver
                {src_info}
                {dst_info}
                old={props.old.clone()}
                new={props.new.clone()}
                path={props.path.clone()}
            />
        }),
        (Err(error), _) | (_, Err(error)) => Ok(html! {
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
    pub src_info: Arc<CrateResponse>,
    pub dst_info: Arc<CrateResponse>,
    pub old: VersionId,
    pub new: VersionId,
    pub path: Option<String>,
}

#[function_component]
pub fn VersionResolver(props: &VersionResolverProps) -> Html {
    // find krate version info
    let old = props.src_info.version(props.old.clone());
    let new = props.dst_info.version(props.new.clone());
    match (old, new) {
        (Some(old), Some(new)) => html! {
            <SourceFetcher
                src_info={props.src_info.clone()}
                dst_info={props.dst_info.clone()}
                old={old.clone()}
                new={new.clone()}
                path={props.path.clone()}
            />
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
    pub src_info: Arc<CrateResponse>,
    pub dst_info: Arc<CrateResponse>,
    pub old: VersionInfo,
    pub new: VersionInfo,
    pub path: Option<String>,
}

#[function_component]
pub fn SourceFetcher(props: &SourceFetcherProps) -> Html {
    let fallback = html! {
        <>
        <ComplexNavbar
            src_name={props.src_info.krate.id.clone()}
            dst_name={props.dst_info.krate.id.clone()}
            old={props.old.num.clone()}
            new={props.new.num.clone()}
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
                    src_name={props.src_info.krate.id.clone()}
                    dst_name={props.dst_info.krate.id.clone()}
                    old={props.old.num.clone()}
                    new={props.new.num.clone()}
                    src_info={props.src_info.clone()}
                    dst_info={props.dst_info.clone()}
                    onchange={
                        let path = props.path.clone();
                        move |((src_name, old), (dst_name, new)): ((String, Version), (String, Version))| {
                            navigator.push(&Route::File {
                                src_name: src_name.clone(),
                                dst_name: dst_name.clone(),
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

    dbg!(&props.path);
    let path = match &props.path {
        None => {
            return Ok(html! {
                <Redirect<Route> to={Route::File {
                    src_name: props.src_info.krate.id.clone(),
                    dst_name: props.dst_info.krate.id.clone(),
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
            src_info={props.src_info.clone()}
            dst_info={props.dst_info.clone()}
            {old}
            {new}
            {path}
        />
    })
}
