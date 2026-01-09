//! Network path analyzer for identifying network segments and issues.

use netdiag_types::diagnostics::{
    BufferBloatGrade, BufferBloatResult, GeoLocation, HealthRating, HopInfo, IspInfo,
    IspServiceType, IssueSeverity, IssueType, LatencyContribution, PathAnalysis, PathHealth,
    PathIssue, PathSegments, SegmentAnalysis, SegmentStatus, SegmentType, TracerouteResult,
};
use std::time::Duration;

/// Path analyzer for comprehensive network path analysis.
pub struct PathAnalyzer {
    /// Threshold for high latency (ms)
    pub high_latency_threshold_ms: u64,
    /// Threshold for latency jump (ms)
    pub latency_jump_threshold_ms: u64,
    /// Threshold for packet loss (percent)
    pub packet_loss_threshold: f64,
}

impl Default for PathAnalyzer {
    fn default() -> Self {
        Self {
            high_latency_threshold_ms: 100,
            latency_jump_threshold_ms: 50,
            packet_loss_threshold: 1.0,
        }
    }
}

impl PathAnalyzer {
    /// Creates a new path analyzer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyzes a traceroute result and produces comprehensive path analysis.
    pub fn analyze(&self, traceroute: &TracerouteResult) -> PathAnalysis {
        let hops = self.extract_hop_info(traceroute);
        let segments = self.identify_segments(&hops);
        let issues = self.identify_issues(&segments, traceroute);
        let health = self.calculate_health(&segments, &issues);
        let recommendations = self.generate_recommendations(&issues);

        PathAnalysis {
            target: traceroute
                .target_hostname
                .clone()
                .unwrap_or_else(|| traceroute.target.to_string()),
            target_ip: Some(traceroute.target),
            segments,
            health,
            issues,
            recommendations,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Extracts hop information from traceroute result.
    fn extract_hop_info(&self, traceroute: &TracerouteResult) -> Vec<HopInfo> {
        traceroute
            .hops
            .iter()
            .map(|hop| HopInfo {
                hop_number: hop.hop,
                ip: hop.address,
                hostname: hop.hostname.clone(),
                rtt: hop.avg_rtt,
                asn: hop.asn,
                as_name: hop.as_name.clone(),
                organization: hop.as_name.clone(), // Use AS name as org for now
                location: hop.location.as_ref().map(|loc| GeoLocation {
                    city: loc.city.clone(),
                    region: None,
                    country: loc.country.clone(),
                    country_code: None,
                    latitude: loc.latitude,
                    longitude: loc.longitude,
                }),
                responsive: !hop.all_timeout,
            })
            .collect()
    }

    /// Identifies network segments from hop information.
    fn identify_segments(&self, hops: &[HopInfo]) -> PathSegments {
        let mut local = SegmentAnalysis::default();
        let mut router = SegmentAnalysis::default();
        let mut isp = SegmentAnalysis::default();
        let mut backbone = SegmentAnalysis::default();
        let mut destination = SegmentAnalysis::default();

        local.segment_type = SegmentType::Local;
        router.segment_type = SegmentType::Router;
        isp.segment_type = SegmentType::Isp;
        backbone.segment_type = SegmentType::Backbone;
        destination.segment_type = SegmentType::Destination;

        // Track unique ASNs to identify segment boundaries
        let mut seen_asns: Vec<Option<u32>> = Vec::new();
        let mut current_segment = SegmentType::Local;

        for hop in hops {
            // Determine which segment this hop belongs to
            let segment = self.classify_hop(hop, &seen_asns, current_segment);
            current_segment = segment;

            // Track ASN changes
            if hop.asn.is_some() && !seen_asns.contains(&hop.asn) {
                seen_asns.push(hop.asn);
            }

            // Add hop to appropriate segment
            match segment {
                SegmentType::Local => local.hops.push(hop.clone()),
                SegmentType::Router => router.hops.push(hop.clone()),
                SegmentType::Isp => isp.hops.push(hop.clone()),
                SegmentType::Backbone => backbone.hops.push(hop.clone()),
                SegmentType::Destination => destination.hops.push(hop.clone()),
                SegmentType::Unknown => {}
            }
        }

        // Calculate latency contributions for each segment
        self.calculate_segment_latency(&mut local, hops);
        self.calculate_segment_latency(&mut router, hops);
        self.calculate_segment_latency(&mut isp, hops);
        self.calculate_segment_latency(&mut backbone, hops);
        self.calculate_segment_latency(&mut destination, hops);

        // Set segment statuses based on analysis
        local.status = self.determine_segment_status(&local);
        router.status = self.determine_segment_status(&router);
        isp.status = self.determine_segment_status(&isp);
        backbone.status = self.determine_segment_status(&backbone);
        destination.status = self.determine_segment_status(&destination);

        PathSegments {
            local,
            router,
            isp,
            backbone,
            destination,
        }
    }

    /// Classifies a hop into a network segment.
    fn classify_hop(
        &self,
        hop: &HopInfo,
        seen_asns: &[Option<u32>],
        current_segment: SegmentType,
    ) -> SegmentType {
        // Hop 1 is always local (default gateway)
        if hop.hop_number == 1 {
            return SegmentType::Local;
        }

        // Hop 2-3 are typically router/access network
        if hop.hop_number <= 3 {
            return SegmentType::Router;
        }

        // Check if this is a new ASN (network boundary)
        let is_new_asn = hop.asn.is_some() && !seen_asns.contains(&hop.asn);

        // Check for destination indicators in hostname
        if let Some(ref hostname) = hop.hostname {
            let hostname_lower = hostname.to_lowercase();

            // Check for backbone indicators
            if hostname_lower.contains("backbone")
                || hostname_lower.contains("core")
                || hostname_lower.contains("bb")
                || hostname_lower.contains("ix")
                || hostname_lower.contains("peer")
            {
                return SegmentType::Backbone;
            }

            // Check for common ISP edge router patterns
            if hop.hop_number <= 6 && !is_new_asn {
                return SegmentType::Isp;
            }
        }

        // Use ASN changes to identify segment boundaries
        match seen_asns.len() {
            0 | 1 => SegmentType::Isp,
            2 => {
                // Second ASN is typically backbone/transit
                if is_new_asn {
                    SegmentType::Backbone
                } else {
                    current_segment
                }
            }
            _ => {
                // Third+ ASN is destination or more backbone
                if is_new_asn {
                    SegmentType::Destination
                } else {
                    current_segment
                }
            }
        }
    }

    /// Calculates latency contribution for a segment.
    fn calculate_segment_latency(&self, segment: &mut SegmentAnalysis, all_hops: &[HopInfo]) {
        if segment.hops.is_empty() {
            return;
        }

        // Get the RTT at entry and exit of segment
        let first_hop_rtt = segment.hops.first().and_then(|h| h.rtt);
        let last_hop_rtt = segment.hops.last().and_then(|h| h.rtt);

        // Find the previous hop's RTT (hop before this segment)
        let prev_hop_rtt = segment.hops.first().and_then(|first| {
            all_hops
                .iter()
                .find(|h| h.hop_number + 1 == first.hop_number)
                .and_then(|h| h.rtt)
        });

        // Calculate segment's latency contribution
        if let (Some(_entry_rtt), Some(exit_rtt)) = (
            prev_hop_rtt.or(first_hop_rtt),
            last_hop_rtt.or(first_hop_rtt),
        ) {
            let segment_latency = exit_rtt.saturating_sub(prev_hop_rtt.unwrap_or(Duration::ZERO));
            let total_latency = all_hops
                .last()
                .and_then(|h| h.rtt)
                .unwrap_or(Duration::ZERO);

            let percentage = if total_latency.as_nanos() > 0 {
                (segment_latency.as_secs_f64() / total_latency.as_secs_f64()) * 100.0
            } else {
                0.0
            };

            segment.latency = Some(LatencyContribution {
                absolute_ms: segment_latency.as_secs_f64() * 1000.0,
                percentage,
                is_primary_contributor: percentage > 40.0,
            });
        }

        // Calculate packet loss for segment
        let total_probes = segment.hops.len();
        let responsive_hops = segment.hops.iter().filter(|h| h.responsive).count();
        segment.packet_loss_percent = if total_probes > 0 {
            ((total_probes - responsive_hops) as f64 / total_probes as f64) * 100.0
        } else {
            0.0
        };
    }

    /// Determines the status of a segment.
    fn determine_segment_status(&self, segment: &SegmentAnalysis) -> SegmentStatus {
        if segment.hops.is_empty() {
            return SegmentStatus::Unknown;
        }

        // Check for complete failure
        if segment.hops.iter().all(|h| !h.responsive) {
            return SegmentStatus::Down;
        }

        let mut issues = 0;

        // Check latency
        if let Some(ref latency) = segment.latency {
            if latency.absolute_ms > self.high_latency_threshold_ms as f64 {
                issues += 2;
            } else if latency.absolute_ms > (self.high_latency_threshold_ms / 2) as f64 {
                issues += 1;
            }
        }

        // Check packet loss
        if segment.packet_loss_percent > self.packet_loss_threshold * 2.0 {
            issues += 2;
        } else if segment.packet_loss_percent > self.packet_loss_threshold {
            issues += 1;
        }

        // Check for unresponsive hops
        let unresponsive_ratio = segment.hops.iter().filter(|h| !h.responsive).count() as f64
            / segment.hops.len() as f64;
        if unresponsive_ratio > 0.5 {
            issues += 2;
        } else if unresponsive_ratio > 0.2 {
            issues += 1;
        }

        match issues {
            0 => SegmentStatus::Healthy,
            1..=2 => SegmentStatus::Degraded,
            3..=4 => SegmentStatus::Impaired,
            _ => SegmentStatus::Down,
        }
    }

    /// Identifies issues in the network path.
    fn identify_issues(
        &self,
        segments: &PathSegments,
        traceroute: &TracerouteResult,
    ) -> Vec<PathIssue> {
        let mut issues = Vec::new();

        // Check each segment for issues
        for (segment_type, segment) in [
            (SegmentType::Local, &segments.local),
            (SegmentType::Router, &segments.router),
            (SegmentType::Isp, &segments.isp),
            (SegmentType::Backbone, &segments.backbone),
            (SegmentType::Destination, &segments.destination),
        ] {
            // High latency in segment
            if let Some(ref latency) = segment.latency {
                if latency.is_primary_contributor && latency.absolute_ms > 100.0 {
                    issues.push(PathIssue {
                        segment: segment_type,
                        issue_type: IssueType::HighLatency,
                        severity: if latency.absolute_ms > 200.0 {
                            IssueSeverity::Error
                        } else {
                            IssueSeverity::Warning
                        },
                        description: format!(
                            "{} segment contributing {:.1}ms ({:.1}% of total latency)",
                            segment_type, latency.absolute_ms, latency.percentage
                        ),
                        details: None,
                        remediation: self.get_latency_remediation(segment_type),
                    });
                }
            }

            // Packet loss in segment
            if segment.packet_loss_percent > self.packet_loss_threshold {
                issues.push(PathIssue {
                    segment: segment_type,
                    issue_type: IssueType::PacketLoss,
                    severity: if segment.packet_loss_percent > 5.0 {
                        IssueSeverity::Error
                    } else {
                        IssueSeverity::Warning
                    },
                    description: format!(
                        "{:.1}% packet loss in {} segment",
                        segment.packet_loss_percent, segment_type
                    ),
                    details: None,
                    remediation: self.get_packet_loss_remediation(segment_type),
                });
            }

            // Segment down
            if segment.status == SegmentStatus::Down {
                issues.push(PathIssue {
                    segment: segment_type,
                    issue_type: IssueType::Unreachable,
                    severity: IssueSeverity::Critical,
                    description: format!("{} segment is unreachable", segment_type),
                    details: None,
                    remediation: Some(
                        "Check physical connectivity and network configuration".to_string(),
                    ),
                });
            }
        }

        // Check for latency jumps between hops
        let jumps = traceroute.latency_jumps(self.latency_jump_threshold_ms);
        for (hop, diff) in jumps {
            issues.push(PathIssue {
                segment: SegmentType::Unknown,
                issue_type: IssueType::LatencySpike,
                severity: if diff.as_millis() > 100 {
                    IssueSeverity::Error
                } else {
                    IssueSeverity::Warning
                },
                description: format!(
                    "Latency spike of {:.1}ms at hop {} ({})",
                    diff.as_millis(),
                    hop.hop,
                    hop.hostname
                        .as_deref()
                        .or(hop.address.map(|a| a.to_string()).as_deref())
                        .unwrap_or("unknown")
                ),
                details: hop.as_name.clone(),
                remediation: None,
            });
        }

        // Check if destination was reached
        if !traceroute.reached {
            issues.push(PathIssue {
                segment: SegmentType::Destination,
                issue_type: IssueType::Unreachable,
                severity: IssueSeverity::Critical,
                description: "Destination unreachable".to_string(),
                details: Some(format!(
                    "Last responding hop: {}",
                    traceroute
                        .hops
                        .iter()
                        .rev()
                        .find(|h| !h.all_timeout)
                        .map(|h| format!("hop {}", h.hop))
                        .unwrap_or_else(|| "none".to_string())
                )),
                remediation: Some(
                    "Destination may be blocking ICMP or experiencing an outage".to_string(),
                ),
            });
        }

        issues
    }

    /// Gets remediation advice for latency issues.
    fn get_latency_remediation(&self, segment: SegmentType) -> Option<String> {
        Some(match segment {
            SegmentType::Local => {
                "Check local network congestion and WiFi signal strength".to_string()
            }
            SegmentType::Router => "Restart router, check for firmware updates".to_string(),
            SegmentType::Isp => {
                "Contact ISP about high latency; check for local outages".to_string()
            }
            SegmentType::Backbone => {
                "Latency may be geographic; consider CDN or different routing".to_string()
            }
            SegmentType::Destination => {
                "Destination server may be overloaded or geographically distant".to_string()
            }
            SegmentType::Unknown => "Unable to determine cause of latency".to_string(),
        })
    }

    /// Gets remediation advice for packet loss issues.
    fn get_packet_loss_remediation(&self, segment: SegmentType) -> Option<String> {
        Some(match segment {
            SegmentType::Local => {
                "Check cables, WiFi interference, or local network congestion".to_string()
            }
            SegmentType::Router => {
                "Restart router, check for overheating or hardware issues".to_string()
            }
            SegmentType::Isp => {
                "Contact ISP; packet loss at ISP level requires provider intervention".to_string()
            }
            SegmentType::Backbone => {
                "Transit network issues; usually resolve automatically".to_string()
            }
            SegmentType::Destination => {
                "Destination network may be congested or experiencing issues".to_string()
            }
            SegmentType::Unknown => "Unable to determine source of packet loss".to_string(),
        })
    }

    /// Calculates overall path health.
    fn calculate_health(&self, segments: &PathSegments, issues: &[PathIssue]) -> PathHealth {
        let mut score = 100u8;

        // Deduct points for issues
        for issue in issues {
            let deduction = match issue.severity {
                IssueSeverity::Info => 2,
                IssueSeverity::Warning => 10,
                IssueSeverity::Error => 25,
                IssueSeverity::Critical => 40,
            };
            score = score.saturating_sub(deduction);
        }

        // Find the most problematic segment
        let problematic_segment = [
            (&segments.local, SegmentType::Local),
            (&segments.router, SegmentType::Router),
            (&segments.isp, SegmentType::Isp),
            (&segments.backbone, SegmentType::Backbone),
            (&segments.destination, SegmentType::Destination),
        ]
        .iter()
        .filter(|(s, _)| matches!(s.status, SegmentStatus::Impaired | SegmentStatus::Down))
        .map(|(_, t)| *t)
        .next();

        let rating = HealthRating::from_score(score);

        let summary = match rating {
            HealthRating::Excellent => {
                "Network path is healthy with no significant issues".to_string()
            }
            HealthRating::Good => "Network path is generally healthy with minor issues".to_string(),
            HealthRating::Fair => {
                "Network path has some issues that may affect performance".to_string()
            }
            HealthRating::Poor => {
                "Network path has significant issues affecting connectivity".to_string()
            }
            HealthRating::Critical => {
                "Network path has critical issues causing severe degradation".to_string()
            }
        };

        PathHealth {
            score,
            rating,
            summary,
            problematic_segment,
        }
    }

    /// Generates recommendations based on identified issues.
    fn generate_recommendations(&self, issues: &[PathIssue]) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Group issues by segment
        let mut has_local_issues = false;
        let mut has_isp_issues = false;
        let mut has_destination_issues = false;

        for issue in issues {
            match issue.segment {
                SegmentType::Local | SegmentType::Router => has_local_issues = true,
                SegmentType::Isp => has_isp_issues = true,
                SegmentType::Destination => has_destination_issues = true,
                _ => {}
            }
        }

        if has_local_issues {
            recommendations.push(
                "Check local network: restart router, verify WiFi signal, check for interference"
                    .to_string(),
            );
        }

        if has_isp_issues {
            recommendations.push(
                "Contact your ISP: report connectivity issues and request a line test".to_string(),
            );
        }

        if has_destination_issues {
            recommendations
                .push("Destination issue: check if the service has reported outages".to_string());
        }

        // Add critical issue recommendations
        for issue in issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
        {
            if let Some(ref remediation) = issue.remediation {
                if !recommendations.contains(remediation) {
                    recommendations.push(remediation.clone());
                }
            }
        }

        if recommendations.is_empty() {
            recommendations.push("No specific actions required at this time".to_string());
        }

        recommendations
    }

    /// Detects buffer bloat by comparing latency under load vs unloaded.
    pub fn detect_buffer_bloat(
        baseline_latency_ms: f64,
        loaded_latency_ms: f64,
    ) -> BufferBloatResult {
        let latency_increase_ms = loaded_latency_ms - baseline_latency_ms;
        let latency_increase_percent = if baseline_latency_ms > 0.0 {
            (latency_increase_ms / baseline_latency_ms) * 100.0
        } else {
            0.0
        };

        let grade = BufferBloatGrade::from_increase_ms(latency_increase_ms);

        let mut recommendations = Vec::new();
        match grade {
            BufferBloatGrade::APlus | BufferBloatGrade::A => {
                recommendations.push("Network handles load well".to_string());
            }
            BufferBloatGrade::B => {
                recommendations
                    .push("Minor buffer bloat detected; consider enabling SQM/QoS".to_string());
            }
            BufferBloatGrade::C => {
                recommendations.push("Moderate buffer bloat; enable SQM/QoS on router".to_string());
                recommendations.push("Consider using fq_codel or cake qdisc".to_string());
            }
            BufferBloatGrade::D | BufferBloatGrade::F => {
                recommendations.push("Severe buffer bloat detected".to_string());
                recommendations.push("Enable SQM with fq_codel or cake immediately".to_string());
                recommendations.push("Consider upgrading router firmware (OpenWrt)".to_string());
            }
        }

        BufferBloatResult {
            baseline_latency_ms,
            loaded_latency_ms,
            latency_increase_ms,
            latency_increase_percent,
            grade,
            recommendations,
        }
    }
}

/// Identifies ISP from traceroute hops.
pub fn identify_isp(hops: &[HopInfo]) -> Option<IspInfo> {
    // Find the first hop with ASN information after the local network
    for hop in hops.iter().skip(2) {
        if let (Some(asn), Some(ref as_name)) = (hop.asn, &hop.as_name) {
            return Some(IspInfo {
                name: as_name.clone(),
                asn: Some(asn),
                service_type: guess_service_type(as_name),
                outage_status: None,
                status_page_url: None,
            });
        }
    }
    None
}

/// Guesses ISP service type from name.
fn guess_service_type(name: &str) -> IspServiceType {
    let lower = name.to_lowercase();

    if lower.contains("fiber") || lower.contains("fios") || lower.contains("ftth") {
        IspServiceType::Fiber
    } else if lower.contains("cable") || lower.contains("comcast") || lower.contains("cox") {
        IspServiceType::Cable
    } else if lower.contains("dsl") || lower.contains("at&t") || lower.contains("centurylink") {
        IspServiceType::Dsl
    } else if lower.contains("wireless") || lower.contains("wisp") {
        IspServiceType::Wireless
    } else if lower.contains("starlink") || lower.contains("satellite") || lower.contains("viasat")
    {
        IspServiceType::Satellite
    } else if lower.contains("mobile")
        || lower.contains("cellular")
        || lower.contains("verizon")
        || lower.contains("t-mobile")
    {
        IspServiceType::Cellular
    } else {
        IspServiceType::Unknown
    }
}
