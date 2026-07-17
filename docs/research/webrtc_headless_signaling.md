# WebRTC Headless Signaling on Local Networks (Rust)

This document covers how to implement headless/direct WebRTC signaling on local networks without standard STUN/TURN servers using the pure Rust `webrtc` library (port of Pion WebRTC).

## 1. Generating a Self-Signed DTLS Certificate

To establish a secure DTLS connection, WebRTC requires a certificate. You can programmatically generate a self-signed certificate using the `rcgen` crate.

```rust
use rcgen::generate_simple_self_signed;

fn generate_certificate() -> Result<rcgen::Certificate, rcgen::RcgenError> {
    // Define the subject alternative names (SANs) for the certificate
    let subject_alt_names = vec!["webrtc.local".to_string()];
    
    // Generate the self-signed certificate
    let cert = generate_simple_self_signed(subject_alt_names)?;
    
    // The `cert` object contains both the private key and public certificate
    Ok(cert)
}
```

*Note: While `webrtc-rs` has built-in helpers like `webrtc::peer_connection::certificate::Certificate::generate_certificate()`, `rcgen` is commonly used when manual control over the certificate generation process is required.*

## 2. Extracting the SHA-256 DTLS Certificate Fingerprint

During the WebRTC signaling phase, peers exchange the SHA-256 fingerprint of the DER-encoded certificate via the SDP offer/answer to verify the DTLS handshake. You can compute this using the `sha2` crate.

```rust
use sha2::{Digest, Sha256};
use rcgen::Certificate;

fn get_dtls_fingerprint(cert: &Certificate) -> String {
    // 1. Serialize the generated certificate to DER format
    let der_cert = cert.serialize_der().expect("Failed to serialize certificate to DER");

    // 2. Hash the DER bytes using SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&der_cert);
    let fingerprint_bytes = hasher.finalize();

    // 3. Format as a colon-separated hexadecimal string (required for SDP)
    let fingerprint_hex: String = fingerprint_bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(":");

    format!("SHA-256 {}", fingerprint_hex)
}
```

## 3. Configuring `APIBuilder` and `SettingEngine` for Local Network Bindings

When operating on local networks without a STUN or TURN server, the ICE agent must rely on direct "host candidates". By default, WebRTC implementations obscure local IPs using mDNS (Multicast DNS) for privacy (generating `.local` addresses). In headless setups, this can cause resolution failures.

To allow direct IP bindings, you must configure the `SettingEngine` to disable mDNS obfuscation, which forces the ICE agent to advertise raw, connectable local IP addresses.

```rust
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_interface::MulticastDnsMode;
use webrtc::ice_transport::ice_network_type::NetworkType;

fn build_webrtc_api() -> webrtc::api::API {
    // 1. Initialize the SettingEngine
    let mut setting_engine = SettingEngine::default();

    // 2. Disable mDNS obfuscation to expose raw local IP addresses
    // This is vital for local peer-to-peer discovery without STUN.
    setting_engine.set_ice_multicast_dns_mode(MulticastDnsMode::Disabled);

    // 3. (Optional) Restrict network interfaces to IPv4 UDP to speed up candidate gathering
    setting_engine.set_network_types(vec![NetworkType::Udp4]);

    // 4. (Optional) If crossing a simple 1:1 NAT, map the local IP to a known direct IP
    // setting_engine.set_nat_1to1_ips(vec!["192.168.1.100".to_owned()], webrtc::ice_transport::ice_candidate_type::ICECandidateType::Host);

    // 5. Build the API with the configured SettingEngine
    APIBuilder::new()
        .with_setting_engine(setting_engine)
        // Add .with_media_engine(...) or .with_interceptor_registry(...) here as needed
        .build()
}
```

With mDNS disabled and an empty list of `ice_servers` in the `RTCConfiguration`, the `PeerConnection` will efficiently establish direct local connections using the provided host candidates.
