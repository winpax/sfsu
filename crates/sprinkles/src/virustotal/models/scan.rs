use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileData {
    #[serde(rename = "type")]
    pub _type: Option<String>,
    pub id: Option<String>,
    pub links: Option<Links>,
    pub data: Option<Data>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    #[serde(rename = "self")]
    pub self_field: Option<String>,
    pub next: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Data {
    pub attributes: Option<Attributes>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    #[serde(rename = "first_seen_itw_date")]
    pub first_seen_itw_date: Option<i64>,
    #[serde(rename = "first_submission_date")]
    pub first_submission_date: Option<i64>,
    #[serde(rename = "last_analysis_date")]
    pub last_analysis_date: Option<i64>,
    #[serde(rename = "last_analysis_results")]
    pub last_analysis_results: Option<HashMap<String, LastAnalysisResults>>,
    #[serde(rename = "last_analysis_stats")]
    pub last_analysis_stats: Option<LastAnalysisStats>,
    #[serde(rename = "last_submission_date")]
    pub last_submission_date: Option<i64>,
    pub magic: Option<String>,
    pub md5: Option<String>,
    pub names: Option<Vec<String>>,
    #[serde(rename = "nsrl_info")]
    pub nsrl_info: Option<NsrlInfo>,
    pub reputation: Option<i64>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub size: Option<i64>,
    pub ssdeep: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(rename = "times_submitted")]
    pub times_submitted: Option<i64>,
    #[serde(rename = "total_votes")]
    pub total_votes: Option<TotalVotes>,
    pub trid: Option<Vec<Trid>>,
    #[serde(rename = "trusted_verdict")]
    pub trusted_verdict: Option<TrustedVerdict>,
    #[serde(rename = "type_description")]
    pub type_description: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastAnalysisResults {
    pub category: Option<String>,
    #[serde(rename = "engine_name")]
    pub engine_name: Option<String>,
    #[serde(rename = "engine_update")]
    pub engine_update: Option<String>,
    #[serde(rename = "engine_version")]
    pub engine_version: Option<String>,
    pub method: Option<String>,
    // TODO: remove serde_json dependency
    pub result: Option<serde_json::Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastAnalysisStats {
    pub harmless: Option<i64>,
    pub malicious: Option<i64>,
    pub suspicious: Option<i64>,
    pub timeout: Option<i64>,
    #[serde(rename = "type-unsupported")]
    pub type_unsupported: Option<i64>,
    pub undetected: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NsrlInfo {
    pub filenames: Option<Vec<String>>,
    pub products: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalVotes {
    pub harmless: Option<i64>,
    pub malicious: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trid {
    pub file_type: Option<String>,
    pub probability: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustedVerdict {
    pub filename: Option<String>,
    pub link: Option<String>,
    pub organization: Option<String>,
    pub verdict: Option<String>,
}
