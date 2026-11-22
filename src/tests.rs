use crate::data::*;
use anyhow::Result;
use serde_json::from_reader;
use std::fs::File;

fn parse_canned_response(name: &str) -> Result<CrateResponse> {
    let response = File::open(format!("data/{name}.json"))?;
    let response: CrateResponse = from_reader(response)?;
    Ok(response)
}

fn parse_canned_source(version: &VersionInfo) -> Result<CrateSource> {
    let data = std::fs::read(format!("data/{}-{}.crate", version.krate, version.version))?;
    let source = CrateSource::new(version.clone(), &data[..])?;
    Ok(source)
}

#[test]
fn test_crate_response_decode_serde() {
    let response = parse_canned_response("serde").unwrap();
    assert_eq!(response.krate.id, "serde");
}

#[test]
fn test_crate_response_decode_axum() {
    let response = parse_canned_response("axum").unwrap();
    assert_eq!(response.krate.id, "axum");
}

#[test]
fn test_crate_response_decode_reqwest() {
    let response = parse_canned_response("reqwest").unwrap();
    assert_eq!(response.krate.id, "reqwest");
}

#[test]
fn test_crate_response_decode_log() {
    let response = parse_canned_response("log").unwrap();
    assert_eq!(response.krate.id, "log");
}

#[test]
fn can_parse_crate_source_log_0_4_15() {
    let log = parse_canned_response("log").unwrap();
    let version = log.version("0.4.15".parse().unwrap()).unwrap();
    let _ = parse_canned_source(version).unwrap();
}

#[test]
fn can_parse_crate_source_log_0_4_16() {
    let log = parse_canned_response("log").unwrap();
    let version = log.version("0.4.16".parse().unwrap()).unwrap();
    let _ = parse_canned_source(version).unwrap();
}

#[test]
fn can_parse_crate_source_log_0_4_17() {
    let log = parse_canned_response("log").unwrap();
    let version = log.version("0.4.17".parse().unwrap()).unwrap();
    let _ = parse_canned_source(version).unwrap();
}

mod routes {
    use crate::Route;
    use yew_router::Routable;

    /// Recognize `path`, upgrade legacy routes, and return the resulting path.
    fn resolve(path: &str) -> String {
        let route = Route::recognize(path).unwrap();
        route.upgrade().unwrap_or(route).to_path()
    }

    #[test]
    fn canonical_routes_are_unchanged() {
        for path in [
            "/serde/1.0.0/1.0.1/files/Cargo.toml",
            "/serde/1.0.0/1.0.1/files/src/de/mod.rs",
            // top-level directory named `files`
            "/serde/1.0.0/1.0.1/files/files/x.rs",
            // top-level directory which parses as a version requirement
            "/serde/1.0.0/1.0.1/files/1/x.rs",
            "/serde/1.0.0/serde_json/1.0.0/src/lib.rs",
            "/serde/1.0.0/serde_json/1.0.0/files/lib.rs",
            "/serde/1.0.0/latest/1.0.0/src/lib.rs",
            "/browse/serde/1.0.0/files/src/lib.rs",
            "/browse/serde/1.0.0/files/files/x.rs",
            "/repo/serde/1.0.0/files/src/lib.rs",
        ] {
            assert_eq!(resolve(path), path);
        }
    }

    #[test]
    fn canonical_routes_match_expected_variant() {
        assert!(matches!(
            Route::recognize("/serde/1.0.0/1.0.1/files/files/x.rs"),
            Some(Route::SingleSourceFile { .. })
        ));
        assert!(matches!(
            Route::recognize("/serde/1.0.0/serde_json/1.0.0/src/lib.rs"),
            Some(Route::File { .. })
        ));
        assert!(matches!(
            Route::recognize("/browse/serde/1.0.0/files/files/x.rs"),
            Some(Route::BrowseFile { .. })
        ));
    }

    #[test]
    fn legacy_routes_are_upgraded() {
        for (legacy, upgraded) in [
            (
                "/serde/1.0.0/1.0.1/Cargo.toml",
                "/serde/1.0.0/1.0.1/files/Cargo.toml",
            ),
            (
                "/serde/1.0.0/1.0.1/src/lib.rs",
                "/serde/1.0.0/1.0.1/files/src/lib.rs",
            ),
            (
                "/serde/1.0.0/1.0.1/src/de/mod.rs",
                "/serde/1.0.0/1.0.1/files/src/de/mod.rs",
            ),
            // first directory must be kept verbatim, not normalized to `^1`
            ("/foo/1.0.0/1.0.1/1/x.rs", "/foo/1.0.0/1.0.1/files/1/x.rs"),
            (
                "/foo/1.0.0/1.0.1/2018/x.rs",
                "/foo/1.0.0/1.0.1/files/2018/x.rs",
            ),
            (
                "/foo/1.0.0/latest/src/lib.rs",
                "/foo/1.0.0/latest/files/src/lib.rs",
            ),
            (
                "/browse/serde/1.0.0/src/lib.rs",
                "/browse/serde/1.0.0/files/src/lib.rs",
            ),
            (
                "/browse/serde/1.0.0/Cargo.toml",
                "/browse/serde/1.0.0/files/Cargo.toml",
            ),
        ] {
            assert_eq!(resolve(legacy), upgraded, "upgrading {legacy}");
        }
    }

    #[test]
    fn invalid_file_routes_are_not_upgraded() {
        let route = Route::recognize("/serde/1.0.0/serde_json/src/lib.rs").unwrap();
        assert!(route.upgrade().is_none());
    }

    #[test]
    fn simplify_uses_files_route() {
        let route = Route::File {
            old_krate: "serde".into(),
            old_version: "1.0.0".parse().unwrap(),
            new_krate: "serde".into(),
            new_version: "1.0.1".parse().unwrap(),
            path: "src/lib.rs".into(),
        };
        assert_eq!(
            route.simplify().to_path(),
            "/serde/1.0.0/1.0.1/files/src/lib.rs"
        );
    }
}
