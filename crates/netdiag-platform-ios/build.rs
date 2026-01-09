//! Build script for netdiag-platform-ios.
//!
//! Links the SystemConfiguration framework for WiFi API access.

fn main() {
    // Only link on iOS targets
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "ios" {
        // Link SystemConfiguration framework for CNCopyCurrentNetworkInfo
        println!("cargo:rustc-link-lib=framework=SystemConfiguration");
    }
}
