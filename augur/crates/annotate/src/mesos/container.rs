use anyhow::Context;
use bstr::BStr;
use reqwest::Client;
use serde::Deserialize;

/// Relevant fields from response when requesting `http://localhost:5051/containers`.
/// This only covers the fields that we actually might care about and has
/// serde_json ignore the rest.  
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct ContainerRecord<'a> {
    pub container_id: &'a BStr,
    pub executor_id: &'a BStr,
    pub executor_name: &'a BStr,
    pub framework_id: &'a BStr,
    pub source: &'a BStr,
}

pub async fn fetch(client: &Client) -> anyhow::Result<Vec<u8>> {
    let body = client
        .get("http://localhost:5051/containers")
        .send()
        .await
        .context("Failed to make a request to http://localhost:5051/containers")?
        .error_for_status()
        .context("Mesos executor returned an error")?
        .bytes()
        .await
        .context("Failed to read request from http://localhost:5051/containers")?
        .to_vec();

    Ok(body)
}

pub fn parse(body: &[u8]) -> anyhow::Result<Vec<ContainerRecord>> {
    serde_json::from_slice(body).context("Mesos executor responded with invalid json")
}
