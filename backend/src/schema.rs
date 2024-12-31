#[derive(Debug)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub account_id: String,
    pub severity: String,
    pub message: String,
    pub acknowledged: bool,
    pub created_at: String,
}

#[derive(Debug)]
pub struct Host {
    pub id: String,
    pub account_id: String,
    pub ip_address: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug)]
pub struct Log {
    pub id: String,
    pub account_id: String,
    pub host_id: String,
    pub version: Option<String>,
    pub device_vendor: Option<String>,
    pub device_product: Option<String>,
    pub device_version: Option<String>,
    pub signature_id: Option<String>,
    pub name: Option<String>,
    pub severity: Option<String>,
    pub extensions: Option<String>,
}

#[derive(Debug)]
pub struct Rule {
    pub id: String,
    pub account_id: String,
    pub title: String,
    pub status: String,
    pub description: String,
    pub references: String,
    pub tags: String,
    pub author: String,
    pub date: String,
    pub logsource: String,
    pub detection: String,
    pub fields: String,
    pub falsepositives: String,
    pub level: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}