//! WiFi security analysis.

use netdiag_types::wifi::{AccessPoint, SecurityType, WifiAuthentication, WifiConnection};
use serde::{Deserialize, Serialize};

/// Security analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    /// Overall security rating.
    pub rating: SecurityRating,

    /// Security score (0-100).
    pub score: u8,

    /// Detected security issues.
    pub issues: Vec<SecurityIssue>,

    /// Security protocol in use.
    pub protocol: SecurityType,

    /// Is enterprise authentication used?
    pub is_enterprise: bool,

    /// Is PMF (Protected Management Frames) enabled?
    pub pmf_enabled: bool,

    /// Recommendations.
    pub recommendations: Vec<String>,
}

/// Security rating levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecurityRating {
    /// Excellent security (WPA3, enterprise).
    Excellent,
    /// Good security (WPA2 with strong config).
    Good,
    /// Fair security (WPA2 personal).
    Fair,
    /// Weak security (WPA or legacy).
    Weak,
    /// No security (open network).
    None,
}

impl SecurityRating {
    /// Get description.
    pub fn description(&self) -> &'static str {
        match self {
            SecurityRating::Excellent => "Excellent - using latest security standards",
            SecurityRating::Good => "Good - secure for most use cases",
            SecurityRating::Fair => "Fair - consider upgrading security",
            SecurityRating::Weak => "Weak - vulnerable to attacks",
            SecurityRating::None => "None - completely open and insecure",
        }
    }

    /// Get score for this rating.
    pub fn score(&self) -> u8 {
        match self {
            SecurityRating::Excellent => 100,
            SecurityRating::Good => 80,
            SecurityRating::Fair => 60,
            SecurityRating::Weak => 40,
            SecurityRating::None => 0,
        }
    }
}

impl std::fmt::Display for SecurityRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SecurityRating::Excellent => "Excellent",
            SecurityRating::Good => "Good",
            SecurityRating::Fair => "Fair",
            SecurityRating::Weak => "Weak",
            SecurityRating::None => "None",
        };
        write!(f, "{}", s)
    }
}

/// Security issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical issue requiring immediate action.
    Critical,
    /// High severity issue.
    High,
    /// Medium severity issue.
    Medium,
    /// Low severity issue.
    Low,
    /// Informational only.
    Info,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            IssueSeverity::Critical => "Critical",
            IssueSeverity::High => "High",
            IssueSeverity::Medium => "Medium",
            IssueSeverity::Low => "Low",
            IssueSeverity::Info => "Info",
        };
        write!(f, "{}", s)
    }
}

/// A detected security issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Issue severity.
    pub severity: IssueSeverity,

    /// Issue title.
    pub title: String,

    /// Detailed description.
    pub description: String,

    /// Remediation steps.
    pub remediation: String,

    /// CVE reference if applicable.
    pub cve: Option<String>,
}

impl SecurityAnalysis {
    /// Analyze security of the current connection.
    pub fn analyze_connection(connection: &WifiConnection) -> Self {
        let protocol = connection.access_point.security.clone();
        let is_enterprise = protocol.is_enterprise();
        let pmf_enabled = protocol.pmf_required;

        let mut issues = Vec::new();

        // Check security protocol based on authentication method
        match protocol.authentication {
            WifiAuthentication::Open => {
                if protocol.is_open() {
                    issues.push(SecurityIssue {
                        severity: IssueSeverity::Critical,
                        title: "Open Network".to_string(),
                        description:
                            "This network has no encryption. All traffic can be intercepted."
                                .to_string(),
                        remediation: "Enable WPA2 or WPA3 encryption on the access point."
                            .to_string(),
                        cve: None,
                    });
                }
            }
            WifiAuthentication::Wep => {
                issues.push(SecurityIssue {
                    severity: IssueSeverity::Critical,
                    title: "WEP Encryption".to_string(),
                    description: "WEP encryption is broken and can be cracked in minutes."
                        .to_string(),
                    remediation: "Upgrade to WPA2 or WPA3 immediately.".to_string(),
                    cve: Some("Multiple".to_string()),
                });
            }
            WifiAuthentication::Wpa => {
                issues.push(SecurityIssue {
                    severity: IssueSeverity::High,
                    title: "WPA (Legacy) Encryption".to_string(),
                    description: "WPA with TKIP has known vulnerabilities.".to_string(),
                    remediation: "Upgrade to WPA2-AES or WPA3.".to_string(),
                    cve: None,
                });
            }
            WifiAuthentication::Wpa2 => {
                if is_enterprise {
                    issues.push(SecurityIssue {
                        severity: IssueSeverity::Low,
                        title: "WPA2 Enterprise".to_string(),
                        description: "WPA2 Enterprise is secure but should verify certificate."
                            .to_string(),
                        remediation: "Ensure RADIUS certificate validation is properly configured."
                            .to_string(),
                        cve: None,
                    });
                } else {
                    // WPA2 Personal
                    issues.push(SecurityIssue {
                        severity: IssueSeverity::Medium,
                        title: "WPA2 Personal".to_string(),
                        description:
                            "WPA2 Personal is secure but may be vulnerable to KRACK if not patched."
                                .to_string(),
                        remediation: "Ensure all devices are patched. Consider upgrading to WPA3."
                            .to_string(),
                        cve: Some("CVE-2017-13077".to_string()),
                    });

                    issues.push(SecurityIssue {
                        severity: IssueSeverity::Info,
                        title: "Password Strength".to_string(),
                        description: "WPA2 Personal security depends on password strength."
                            .to_string(),
                        remediation: "Use a password of at least 12 characters with mixed case, numbers, and symbols.".to_string(),
                        cve: None,
                    });
                }
            }
            WifiAuthentication::Wpa3 => {
                issues.push(SecurityIssue {
                    severity: IssueSeverity::Info,
                    title: "WPA3 Encryption".to_string(),
                    description: "Using the latest WiFi security standard with SAE authentication."
                        .to_string(),
                    remediation: "No action needed - you're using best practices.".to_string(),
                    cve: None,
                });
            }
            WifiAuthentication::Owe => {
                issues.push(SecurityIssue {
                    severity: IssueSeverity::Low,
                    title: "OWE (Enhanced Open)".to_string(),
                    description: "Using Opportunistic Wireless Encryption for open networks."
                        .to_string(),
                    remediation: "Good for public networks, consider WPA3 for private networks."
                        .to_string(),
                    cve: None,
                });
            }
        }

        // Check PMF
        if !pmf_enabled && protocol.authentication == WifiAuthentication::Wpa2 {
            issues.push(SecurityIssue {
                severity: IssueSeverity::Low,
                title: "PMF Not Required".to_string(),
                description: "Protected Management Frames help prevent deauthentication attacks."
                    .to_string(),
                remediation: "Enable PMF (802.11w) on the access point if supported.".to_string(),
                cve: None,
            });
        }

        // Calculate rating and score
        let rating = Self::calculate_rating(&protocol);
        let score = Self::calculate_score(&issues, &rating);

        // Generate recommendations
        let recommendations = Self::generate_recommendations(&issues, &protocol);

        Self {
            rating,
            score,
            issues,
            protocol,
            is_enterprise,
            pmf_enabled,
            recommendations,
        }
    }

    /// Analyze security of a scanned access point.
    pub fn analyze_ap(ap: &AccessPoint) -> Self {
        let protocol = ap.security.clone();
        let is_enterprise = protocol.is_enterprise();
        let pmf_enabled = protocol.pmf_required;

        let mut issues = Vec::new();

        if protocol.is_open() {
            issues.push(SecurityIssue {
                severity: IssueSeverity::Critical,
                title: "Open Network".to_string(),
                description: format!("Network '{}' is open with no encryption", ap.ssid),
                remediation: "Do not connect or use VPN if you must connect.".to_string(),
                cve: None,
            });
        } else if protocol.authentication == WifiAuthentication::Wep {
            issues.push(SecurityIssue {
                severity: IssueSeverity::Critical,
                title: "WEP Network".to_string(),
                description: format!("Network '{}' uses broken WEP encryption", ap.ssid),
                remediation: "Avoid this network entirely.".to_string(),
                cve: None,
            });
        }

        let rating = Self::calculate_rating(&protocol);
        let score = Self::calculate_score(&issues, &rating);
        let recommendations = Self::generate_recommendations(&issues, &protocol);

        Self {
            rating,
            score,
            issues,
            protocol,
            is_enterprise,
            pmf_enabled,
            recommendations,
        }
    }

    /// Calculate security rating.
    fn calculate_rating(protocol: &SecurityType) -> SecurityRating {
        if protocol.is_open() {
            return SecurityRating::None;
        }

        match protocol.authentication {
            WifiAuthentication::Wep => SecurityRating::Weak,
            WifiAuthentication::Wpa => SecurityRating::Weak,
            WifiAuthentication::Wpa2 => {
                if protocol.is_enterprise() {
                    SecurityRating::Good
                } else if protocol.pmf_required {
                    SecurityRating::Good
                } else {
                    SecurityRating::Fair
                }
            }
            WifiAuthentication::Wpa3 => SecurityRating::Excellent,
            WifiAuthentication::Owe => SecurityRating::Fair,
            WifiAuthentication::Open => SecurityRating::None,
        }
    }

    /// Calculate security score.
    fn calculate_score(issues: &[SecurityIssue], rating: &SecurityRating) -> u8 {
        let base_score = rating.score();

        // Deduct for issues
        let deductions: u8 = issues
            .iter()
            .map(|i| match i.severity {
                IssueSeverity::Critical => 30,
                IssueSeverity::High => 20,
                IssueSeverity::Medium => 10,
                IssueSeverity::Low => 5,
                IssueSeverity::Info => 0,
            })
            .sum();

        base_score.saturating_sub(deductions)
    }

    /// Generate recommendations.
    fn generate_recommendations(issues: &[SecurityIssue], protocol: &SecurityType) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Add issue-specific recommendations
        for issue in issues {
            if issue.severity <= IssueSeverity::Medium && !issue.remediation.is_empty() {
                recommendations.push(issue.remediation.clone());
            }
        }

        // Add general recommendations based on security level
        let level = protocol.security_level();
        if level <= 1 {
            recommendations.push("Use a VPN when connected to untrusted networks.".to_string());
        } else if level == 2 || level == 3 && !protocol.is_enterprise() {
            recommendations.push("Consider upgrading to WPA3 for enhanced security.".to_string());
        }

        // Deduplicate
        recommendations.sort();
        recommendations.dedup();

        recommendations
    }

    /// Check if the connection is considered secure.
    pub fn is_secure(&self) -> bool {
        self.rating >= SecurityRating::Fair
    }

    /// Get critical issues.
    pub fn critical_issues(&self) -> Vec<&SecurityIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .collect()
    }
}
