use std::collections::HashMap;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Event {
    pub event_ts: u64,
    pub event_type: EventType,
    pub expires_ts: Option<u64>,
    pub fetch_status: Option<HashMap<WxApp, u16>>,
    pub image_uri: Option<String>,
    pub ingest_ts: u64,
    pub location: Option<Location>,
    pub md: Option<MesoscaleDiscussion>,
    pub outlook: Option<Outlook>,
    pub report: Option<Report>,
    pub text: Option<String>,
    pub title: String, // Max 31 chars
    pub valid_ts: Option<u64>,
    pub warning: Option<Warning>,
    pub watch: Option<Watch>,
}

impl Event {
    pub fn new(event_ts: u64, event_type: EventType, title: String) -> Event {
        Event {
            event_ts,
            event_type,
            expires_ts: None,
            fetch_status: None,
            image_uri: None,
            ingest_ts: 0,
            location: None,
            md: None,
            outlook: None,
            report: None,
            text: None,
            title,
            valid_ts: None,
            warning: None,
            watch: None,
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EventType {
    SnReport,
    SpcSfcoa,
    NwsAfd,
    NwsLsr,
    NwsSel,
    NwsSvr,
    NwsSvs,
    NwsSwo,
    NwsTor,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Location {
    pub wfo: Option<String>,
    pub point: Option<Coordinates>,
    pub poly: Option<Vec<Coordinates>>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Coordinates {
    pub lat: f32,
    pub lon: f32,
}

/**
 * Contains most possible permutations of SN reports and LSRs.
 */
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum HazardType {
    Tornado = 0isize,
    Funnel,
    WallCloud,
    Hail,
    Wind,
    Flood,
    FlashFlood,
    Other,
    FreezingRain,
    Snow,
    Downburst,
    HeavyRain,
    MarineWind,
    Lightning,
    Waterspout,
    Wildfire
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Report {
    pub reporter: String,
    pub hazard: HazardType,
    pub magnitude: Option<f32>,
    pub units: Option<Units>,
    pub was_measured: Option<bool>,
    pub report_ts: Option<u64>, // only populated for LSRs
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Units {
    Knots,
    Mph,
    Inches,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Watch {
    pub is_pds: bool,
    pub id: u16,
    pub watch_type: WatchType,
    pub status: WatchStatus,
    pub issued_for: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WatchType {
    Tornado,
    SevereThunderstorm,
    Other,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WatchStatus {
    Issued,
    Cancelled,
    Unknown,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Warning {
    pub is_pds: bool,
    pub is_tor_emergency: bool, // TOR only
    pub was_observed: bool,     // TOR only
    pub issued_for: String,
    pub motion_deg: u16, // TOR only
    pub motion_kt: u16,  // TOR only
    pub source: String,
    pub time: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Outlook {
    pub swo_type: SwoType,
    pub max_risk: OutlookRisk,
    pub polys: Option<HashMap<OutlookRisk, Vec<Coordinates>>>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SwoType {
    Day1,
    Day2,
    Day3,
    Day48,
    MesoscaleDiscussion,
    Unknown,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum OutlookRisk {
    TSTM,
    MRGL,
    SLGT,
    ENH,
    MDT,
    HIGH,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MdConcerning {
    ExistingTorWatch,
    ExistingSvrWatch,
    NewTorWatch,
    NewSvrWatch,
    Unknown,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MesoscaleDiscussion {
    pub id: u16,
    pub affected: String,
    pub concerning: MdConcerning,
    pub watch_issuance_probability: Option<u16>,
    pub wfos: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FetchFailure {
    pub app: WxApp,
    pub ingest_ts: u64,
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum WxApp {
    SpotterNetworkLoader,
    NwsApiLoader,
    SpcSfcoaLoader,
    FetchFailureLoader,
    Admin,
}
