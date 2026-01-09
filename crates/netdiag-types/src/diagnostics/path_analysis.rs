//! Network path analysis types for identifying issues at different network segments.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Comprehensive network path analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathAnalysis {
    /// Target being analyzed
    pub target: String,
    /// Resolved target IP
    pub target_ip: Option<IpAddr>,
    /// Analysis of each network segment
    pub segments: PathSegments,
    /// Overall path health
    pub health: PathHealth,
    /// Identified issues
    pub issues: Vec<PathIssue>,
    /// Recommendations for fixing issues
    pub recommendations: Vec<String>,
    /// Analysis timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Network segments in the path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSegments {
    /// Local network segment (device to router)
    pub local: SegmentAnalysis,
    /// Router/gateway segment
    pub router: SegmentAnalysis,
    /// ISP segment (first mile)
    pub isp: SegmentAnalysis,
    /// Backbone/transit segment
    pub backbone: SegmentAnalysis,
    /// Destination network segment
    pub destination: SegmentAnalysis,
}

/// Analysis of a single network segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentAnalysis {
    /// Segment type
    pub segment_type: SegmentType,
    /// Status of this segment
    pub status: SegmentStatus,
    /// Hops in this segment
    pub hops: Vec<HopInfo>,
    /// Latency contribution of this segment
    pub latency: Option<LatencyContribution>,
    /// Packet loss in this segment
    pub packet_loss_percent: f64,
    /// Network owner information
    pub network_owner: Option<NetworkOwner>,
    /// Detected issues in this segment
    pub issues: Vec<SegmentIssue>,
}

impl Default for SegmentAnalysis {
    fn default() -> Self {
        Self {
            segment_type: SegmentType::Unknown,
            status: SegmentStatus::Unknown,
            hops: Vec::new(),
            latency: None,
            packet_loss_percent: 0.0,
            network_owner: None,
            issues: Vec::new(),
        }
    }
}

/// Type of network segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, Default)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    /// Local network (device to first hop)
    Local,
    /// Router/gateway
    Router,
    /// ISP network (first mile)
    Isp,
    /// Backbone/transit network
    Backbone,
    /// Destination network
    Destination,
    /// Unknown segment
    #[default]
    Unknown,
}

/// Status of a network segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, Default)]
#[serde(rename_all = "snake_case")]
pub enum SegmentStatus {
    /// Segment is healthy
    Healthy,
    /// Segment has minor issues
    Degraded,
    /// Segment has significant issues
    Impaired,
    /// Segment is down or unreachable
    Down,
    /// Status unknown
    #[default]
    Unknown,
}

/// Information about a hop in the path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopInfo {
    /// Hop number
    pub hop_number: u8,
    /// IP address
    pub ip: Option<IpAddr>,
    /// Hostname
    pub hostname: Option<String>,
    /// Round-trip time
    pub rtt: Option<Duration>,
    /// ASN
    pub asn: Option<u32>,
    /// AS name
    pub as_name: Option<String>,
    /// Organization/ISP name
    pub organization: Option<String>,
    /// Geographic location
    pub location: Option<GeoLocation>,
    /// Whether this hop is responsive
    pub responsive: bool,
}

/// Geographic location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// City
    pub city: Option<String>,
    /// Region/state
    pub region: Option<String>,
    /// Country
    pub country: Option<String>,
    /// Country code (ISO 3166-1 alpha-2)
    pub country_code: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
}

/// Latency contribution of a segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyContribution {
    /// Absolute latency added by this segment
    pub absolute_ms: f64,
    /// Percentage of total latency
    pub percentage: f64,
    /// Is this the primary latency contributor?
    pub is_primary_contributor: bool,
}

/// Network owner information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOwner {
    /// Organization name
    pub name: String,
    /// ASN (Autonomous System Number)
    pub asn: Option<u32>,
    /// Network type
    pub network_type: NetworkType,
    /// WHOIS registry (ARIN, RIPE, APNIC, etc.)
    pub registry: Option<String>,
    /// Network range
    pub network_range: Option<String>,
}

/// Type of network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, Default)]
#[serde(rename_all = "snake_case")]
pub enum NetworkType {
    /// Residential ISP
    ResidentialIsp,
    /// Business ISP
    BusinessIsp,
    /// Tier 1 backbone provider
    Tier1Backbone,
    /// Tier 2 transit provider
    Tier2Transit,
    /// Content Delivery Network
    Cdn,
    /// Cloud provider
    CloudProvider,
    /// Hosting provider
    HostingProvider,
    /// Enterprise network
    Enterprise,
    /// Educational/research network
    Educational,
    /// Government network
    Government,
    /// Unknown type
    #[default]
    Unknown,
}

/// Issue detected in a segment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentIssue {
    /// Issue type
    pub issue_type: IssueType,
    /// Severity
    pub severity: IssueSeverity,
    /// Description
    pub description: String,
    /// Affected hop(s)
    pub affected_hops: Vec<u8>,
}

/// Type of path issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    /// High latency
    HighLatency,
    /// Latency spike/jump
    LatencySpike,
    /// Packet loss
    PacketLoss,
    /// Unreachable hop
    Unreachable,
    /// Route instability
    RouteInstability,
    /// Congestion detected
    Congestion,
    /// Possible outage
    PossibleOutage,
    /// Geographic routing anomaly
    RoutingAnomaly,
    /// MTU issue
    MtuIssue,
    /// DNS resolution failure
    DnsFailure,
}

/// Severity of an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    /// Informational
    Info,
    /// Warning - may cause minor issues
    Warning,
    /// Error - causing noticeable problems
    Error,
    /// Critical - causing major disruption
    Critical,
}

/// Overall path health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathHealth {
    /// Overall score (0-100)
    pub score: u8,
    /// Health rating
    pub rating: HealthRating,
    /// Summary
    pub summary: String,
    /// Problematic segment (if any)
    pub problematic_segment: Option<SegmentType>,
}

/// Health rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum HealthRating {
    /// Excellent health (90-100)
    Excellent,
    /// Good health (70-89)
    Good,
    /// Fair health (50-69)
    Fair,
    /// Poor health (30-49)
    Poor,
    /// Critical issues (0-29)
    Critical,
}

impl HealthRating {
    /// Determines rating from score.
    #[must_use]
    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => Self::Excellent,
            70..=89 => Self::Good,
            50..=69 => Self::Fair,
            30..=49 => Self::Poor,
            _ => Self::Critical,
        }
    }
}

/// Identified path issue with context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathIssue {
    /// Segment where issue occurs
    pub segment: SegmentType,
    /// Issue type
    pub issue_type: IssueType,
    /// Severity
    pub severity: IssueSeverity,
    /// Human-readable description
    pub description: String,
    /// Technical details
    pub details: Option<String>,
    /// Suggested remediation
    pub remediation: Option<String>,
}

/// ISP information with outage status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IspInfo {
    /// ISP name
    pub name: String,
    /// ASN
    pub asn: Option<u32>,
    /// Type of service
    pub service_type: IspServiceType,
    /// Known outage status
    pub outage_status: Option<OutageStatus>,
    /// ISP status page URL
    pub status_page_url: Option<String>,
}

/// ISP service type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum IspServiceType {
    /// Fiber connection
    Fiber,
    /// Cable connection
    Cable,
    /// DSL connection
    Dsl,
    /// Wireless/fixed wireless
    Wireless,
    /// Satellite
    Satellite,
    /// Cellular/mobile
    Cellular,
    /// Business leased line
    LeasedLine,
    /// Unknown type
    Unknown,
}

/// Outage status for a service/ISP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutageStatus {
    /// Whether an outage is currently active
    pub is_outage: bool,
    /// Outage severity
    pub severity: OutageSeverity,
    /// Affected region/area
    pub affected_area: Option<String>,
    /// Estimated time to resolution
    pub estimated_resolution: Option<String>,
    /// Source of outage information
    pub source: String,
    /// Last updated
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Description
    pub description: Option<String>,
}

/// Outage severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum OutageSeverity {
    /// No outage
    None,
    /// Minor/localized issues
    Minor,
    /// Partial outage
    Partial,
    /// Major outage
    Major,
    /// Complete outage
    Complete,
}

/// Buffer bloat detection result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferBloatResult {
    /// Baseline latency (unloaded)
    pub baseline_latency_ms: f64,
    /// Latency under load
    pub loaded_latency_ms: f64,
    /// Latency increase
    pub latency_increase_ms: f64,
    /// Latency increase percentage
    pub latency_increase_percent: f64,
    /// Buffer bloat grade
    pub grade: BufferBloatGrade,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Buffer bloat grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum BufferBloatGrade {
    /// Excellent (< 5ms increase)
    #[strum(serialize = "A+")]
    APlus,
    /// Very good (5-30ms increase)
    A,
    /// Good (30-60ms increase)
    B,
    /// Fair (60-200ms increase)
    C,
    /// Poor (200-400ms increase)
    D,
    /// Very poor (> 400ms increase)
    F,
}

impl BufferBloatGrade {
    /// Determines grade from latency increase.
    #[must_use]
    pub fn from_increase_ms(increase_ms: f64) -> Self {
        if increase_ms < 5.0 {
            Self::APlus
        } else if increase_ms < 30.0 {
            Self::A
        } else if increase_ms < 60.0 {
            Self::B
        } else if increase_ms < 200.0 {
            Self::C
        } else if increase_ms < 400.0 {
            Self::D
        } else {
            Self::F
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_rating_from_score() {
        assert_eq!(HealthRating::from_score(100), HealthRating::Excellent);
        assert_eq!(HealthRating::from_score(95), HealthRating::Excellent);
        assert_eq!(HealthRating::from_score(90), HealthRating::Excellent);
        assert_eq!(HealthRating::from_score(89), HealthRating::Good);
        assert_eq!(HealthRating::from_score(75), HealthRating::Good);
        assert_eq!(HealthRating::from_score(70), HealthRating::Good);
        assert_eq!(HealthRating::from_score(69), HealthRating::Fair);
        assert_eq!(HealthRating::from_score(55), HealthRating::Fair);
        assert_eq!(HealthRating::from_score(50), HealthRating::Fair);
        assert_eq!(HealthRating::from_score(49), HealthRating::Poor);
        assert_eq!(HealthRating::from_score(35), HealthRating::Poor);
        assert_eq!(HealthRating::from_score(30), HealthRating::Poor);
        assert_eq!(HealthRating::from_score(29), HealthRating::Critical);
        assert_eq!(HealthRating::from_score(10), HealthRating::Critical);
        assert_eq!(HealthRating::from_score(0), HealthRating::Critical);
    }

    #[test]
    fn test_buffer_bloat_grade_from_increase() {
        assert_eq!(BufferBloatGrade::from_increase_ms(0.0), BufferBloatGrade::APlus);
        assert_eq!(BufferBloatGrade::from_increase_ms(4.9), BufferBloatGrade::APlus);
        assert_eq!(BufferBloatGrade::from_increase_ms(5.0), BufferBloatGrade::A);
        assert_eq!(BufferBloatGrade::from_increase_ms(29.9), BufferBloatGrade::A);
        assert_eq!(BufferBloatGrade::from_increase_ms(30.0), BufferBloatGrade::B);
        assert_eq!(BufferBloatGrade::from_increase_ms(59.9), BufferBloatGrade::B);
        assert_eq!(BufferBloatGrade::from_increase_ms(60.0), BufferBloatGrade::C);
        assert_eq!(BufferBloatGrade::from_increase_ms(199.9), BufferBloatGrade::C);
        assert_eq!(BufferBloatGrade::from_increase_ms(200.0), BufferBloatGrade::D);
        assert_eq!(BufferBloatGrade::from_increase_ms(399.9), BufferBloatGrade::D);
        assert_eq!(BufferBloatGrade::from_increase_ms(400.0), BufferBloatGrade::F);
        assert_eq!(BufferBloatGrade::from_increase_ms(1000.0), BufferBloatGrade::F);
    }

    #[test]
    fn test_segment_analysis_default() {
        let segment = SegmentAnalysis::default();
        assert_eq!(segment.segment_type, SegmentType::Unknown);
        assert_eq!(segment.status, SegmentStatus::Unknown);
        assert!(segment.hops.is_empty());
        assert!(segment.latency.is_none());
        assert_eq!(segment.packet_loss_percent, 0.0);
        assert!(segment.network_owner.is_none());
        assert!(segment.issues.is_empty());
    }

    #[test]
    fn test_segment_type_default() {
        assert_eq!(SegmentType::default(), SegmentType::Unknown);
    }

    #[test]
    fn test_segment_status_default() {
        assert_eq!(SegmentStatus::default(), SegmentStatus::Unknown);
    }

    #[test]
    fn test_network_type_default() {
        assert_eq!(NetworkType::default(), NetworkType::Unknown);
    }

    #[test]
    fn test_hop_info_creation() {
        let hop = HopInfo {
            hop_number: 1,
            ip: Some("192.168.1.1".parse().unwrap()),
            hostname: Some("router.local".to_string()),
            rtt: Some(Duration::from_millis(5)),
            asn: None,
            as_name: None,
            organization: None,
            location: None,
            responsive: true,
        };

        assert_eq!(hop.hop_number, 1);
        assert!(hop.ip.is_some());
        assert!(hop.responsive);
        assert_eq!(hop.rtt, Some(Duration::from_millis(5)));
    }

    #[test]
    fn test_latency_contribution() {
        let latency = LatencyContribution {
            absolute_ms: 15.0,
            percentage: 30.0,
            is_primary_contributor: true,
        };

        assert_eq!(latency.absolute_ms, 15.0);
        assert_eq!(latency.percentage, 30.0);
        assert!(latency.is_primary_contributor);
    }

    #[test]
    fn test_path_issue_severity_ordering() {
        assert!(IssueSeverity::Info < IssueSeverity::Warning);
        assert!(IssueSeverity::Warning < IssueSeverity::Error);
        assert!(IssueSeverity::Error < IssueSeverity::Critical);
    }

    #[test]
    fn test_geo_location() {
        let location = GeoLocation {
            city: Some("San Francisco".to_string()),
            region: Some("California".to_string()),
            country: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            latitude: Some(37.7749),
            longitude: Some(-122.4194),
        };

        assert_eq!(location.city, Some("San Francisco".to_string()));
        assert_eq!(location.country_code, Some("US".to_string()));
    }

    #[test]
    fn test_outage_status() {
        let outage = OutageStatus {
            is_outage: true,
            severity: OutageSeverity::Major,
            affected_area: Some("Bay Area".to_string()),
            estimated_resolution: Some("2 hours".to_string()),
            source: "Status Page".to_string(),
            last_updated: Some(chrono::Utc::now()),
            description: Some("Network maintenance".to_string()),
        };

        assert!(outage.is_outage);
        assert_eq!(outage.severity, OutageSeverity::Major);
    }

    #[test]
    fn test_buffer_bloat_result() {
        let baseline = 20.0;
        let loaded = 150.0;
        let increase = loaded - baseline;
        let increase_percent = (increase / baseline) * 100.0;

        let result = BufferBloatResult {
            baseline_latency_ms: baseline,
            loaded_latency_ms: loaded,
            latency_increase_ms: increase,
            latency_increase_percent: increase_percent,
            grade: BufferBloatGrade::from_increase_ms(increase),
            recommendations: vec!["Enable SQM/fq_codel".to_string()],
        };

        assert_eq!(result.baseline_latency_ms, 20.0);
        assert_eq!(result.loaded_latency_ms, 150.0);
        assert_eq!(result.latency_increase_ms, 130.0);
        assert_eq!(result.grade, BufferBloatGrade::C);
    }
}
