use serde::{Deserialize, Serialize};

/// Requête envoyée par le client
#[derive(Debug, Deserialize)]
pub struct Request {
    pub cmd: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub seconds: Option<u64>,
}

/// Réponse envoyée au client
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Response {
    Ok {
        status: String,
    },
    OkWithValue {
        status: String,
        value: Option<String>,
    },
    OkWithCount {
        status: String,
        count: u32,
    },
    OkWithKeys {
        status: String,
        keys: Vec<String>,
    },
    OkWithTtl {
        status: String,
        ttl: i64,
    },
    OkWithIntValue {
        status: String,
        value: i64,
    },
    Error {
        status: String,
        message: String,
    },
}

impl Response {
    pub fn ok() -> Self {
        Response::Ok {
            status: "ok".to_string(),
        }
    }

    pub fn ok_with_value(value: Option<String>) -> Self {
        Response::OkWithValue {
            status: "ok".to_string(),
            value,
        }
    }

    pub fn ok_with_count(count: u32) -> Self {
        Response::OkWithCount {
            status: "ok".to_string(),
            count,
        }
    }

    pub fn ok_with_keys(keys: Vec<String>) -> Self {
        Response::OkWithKeys {
            status: "ok".to_string(),
            keys,
        }
    }

    pub fn ok_with_ttl(ttl: i64) -> Self {
        Response::OkWithTtl {
            status: "ok".to_string(),
            ttl,
        }
    }

    pub fn ok_with_int_value(value: i64) -> Self {
        Response::OkWithIntValue {
            status: "ok".to_string(),
            value,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Response::Error {
            status: "error".to_string(),
            message: message.into(),
        }
    }
}
