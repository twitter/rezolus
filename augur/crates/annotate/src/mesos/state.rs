use anyhow::Context;
use bstr::BStr;
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(bound = "'de: 'a")]
pub struct State<'a> {
    pub frameworks: Vec<Framework<'a>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(bound = "'de: 'a")]
pub struct Framework<'a> {
    pub executors: Vec<Executor<'a>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(bound = "'de: 'a")]
pub struct Executor<'a> {
    pub id: &'a BStr,
    pub source: &'a BStr,
    pub tasks: Vec<Task<'a>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(bound = "'de: 'a")]
pub struct Task<'a> {
    pub id: &'a BStr,
    pub name: &'a BStr,
    pub executor_id: &'a BStr,
}

pub async fn fetch(client: &Client) -> anyhow::Result<Vec<u8>> {
    let body = client
        .get("http://localhost:5051/state")
        .send()
        .await
        .context("Failed to make a request to http://localhost:5051/state")?
        .error_for_status()
        .context("Mesos executor returned an error")?
        .bytes()
        .await
        .context("Failed to read request from http://localhost:5051/state")?
        .to_vec();
    Ok(body)
}

pub fn parse(body: &[u8]) -> anyhow::Result<State> {
    serde_json::from_slice(body).context("Mesos executor responded with invalid json")
}
