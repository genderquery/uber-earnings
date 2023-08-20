use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

mod iso_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

mod utc_timestamp {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = date.timestamp();
        serializer.serialize_i64(timestamp)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp = i64::deserialize(deserializer)?;
        Utc.timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| serde::de::Error::custom("out-of-range number of seconds"))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityRequest {
    #[serde(with = "iso_date_format")]
    pub start_date_iso: NaiveDate,
    #[serde(with = "iso_date_format")]
    pub end_date_iso: NaiveDate,
    pub pagination_option: Option<PaginationOption>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOption {
    pub cursor: String,
}

#[derive(Deserialize)]
#[serde(tag = "status")]
#[serde(rename_all = "camelCase")]
pub enum ActivityFeedResponse {
    Failure { data: FailureData },
    Success { data: SuccessData },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureData {
    pub message: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessData {
    pub activities: Option<Vec<Activity>>,
    pub pagination: Pagination,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub uuid: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(with = "utc_timestamp")]
    pub recognized_at: DateTime<Utc>,
    pub activity_title: String,
    pub formatted_total: String,
    pub routing: Routing,
    pub breakdown_details: Option<BreakdownDetails>,
    pub trip_meta_data: Option<TripMetaData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Routing {
    pub webview_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakdownDetails {
    pub formatted_tip: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripMetaData {
    pub formatted_duration: String,
    pub formatted_distance: String,
    pub pickup_address: String,
    pub drop_off_address: String,
    pub map_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub has_more_data: bool,
    pub next_cursor: Option<String>,
}
