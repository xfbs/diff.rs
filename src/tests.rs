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
