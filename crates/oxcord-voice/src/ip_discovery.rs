use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::error::{Result, VoiceError};

/// Size of the Discord IP Discovery packet in bytes.
pub const IP_DISCOVERY_PACKET_SIZE: usize = 74;

/// Generates a 74-byte Discord IP Discovery request packet for a given SSRC.
#[must_use]
pub fn create_ip_discovery_request(ssrc: u32) -> [u8; IP_DISCOVERY_PACKET_SIZE] {
    let mut packet = [0u8; IP_DISCOVERY_PACKET_SIZE];
    // Type 1 (Request): 2 bytes big-endian
    packet[0..2].copy_from_slice(&1u16.to_be_bytes());
    // Length: 70 bytes
    packet[2..4].copy_from_slice(&70u16.to_be_bytes());
    // SSRC: 4 bytes big-endian
    packet[4..8].copy_from_slice(&ssrc.to_be_bytes());
    packet
}

/// Parses a 74-byte Discord IP Discovery response packet, returning the external `(IpAddr, port)`.
///
/// # Errors
/// Returns [`VoiceError::IpDiscovery`] if the packet format or IP address is invalid.
pub fn parse_ip_discovery_response(packet: &[u8]) -> Result<(IpAddr, u16)> {
    if packet.len() < IP_DISCOVERY_PACKET_SIZE {
        return Err(VoiceError::IpDiscovery(format!(
            "response packet too short: expected {}, got {}",
            IP_DISCOVERY_PACKET_SIZE,
            packet.len()
        )));
    }

    let msg_type = u16::from_be_bytes([packet[0], packet[1]]);
    if msg_type != 2 {
        return Err(VoiceError::IpDiscovery(format!(
            "unexpected response type: expected 2, got {msg_type}"
        )));
    }

    // IP address is stored in bytes 8..72 as a null-terminated ASCII string
    let ip_bytes = &packet[8..72];
    let null_pos = ip_bytes.iter().position(|&b| b == 0).unwrap_or(ip_bytes.len());

    let ip_str = std::str::from_utf8(&ip_bytes[..null_pos])
        .map_err(|e| VoiceError::IpDiscovery(format!("invalid UTF-8 in IP address: {e}")))?;

    let ip = IpAddr::from_str(ip_str).map_err(|e| {
        VoiceError::IpDiscovery(format!("failed to parse discovered IP '{ip_str}': {e}"))
    })?;

    // Port is stored in bytes 72..74 as a big-endian u16
    let port = u16::from_be_bytes([packet[72], packet[73]]);

    Ok((ip, port))
}

/// Performs Discord IP Discovery over the given UDP socket to discover the client's external IP and
/// port.
///
/// Sends an IP Discovery request to `server_addr` and awaits a response within `timeout_dur`.
///
/// # Errors
/// Returns [`VoiceError`] if sending/receiving fails or times out.
pub async fn perform_ip_discovery(
    socket: &UdpSocket,
    server_addr: SocketAddr,
    ssrc: u32,
    timeout_dur: Duration,
) -> Result<(IpAddr, u16)> {
    let request = create_ip_discovery_request(ssrc);
    socket.send_to(&request, server_addr).await?;

    let mut buf = [0u8; IP_DISCOVERY_PACKET_SIZE];
    let (len, _src) = timeout(timeout_dur, socket.recv_from(&mut buf)).await.map_err(|_| {
        VoiceError::IpDiscovery("timed out waiting for IP discovery response".into())
    })??;

    parse_ip_discovery_response(&buf[..len])
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn test_ip_discovery_request_and_response() {
        let ssrc = 0x1234_5678;
        let request = create_ip_discovery_request(ssrc);

        assert_eq!(&request[0..2], &1u16.to_be_bytes());
        assert_eq!(&request[2..4], &70u16.to_be_bytes());
        assert_eq!(&request[4..8], &ssrc.to_be_bytes());

        // Construct mock response packet
        let mut response = [0u8; IP_DISCOVERY_PACKET_SIZE];
        response[0..2].copy_from_slice(&2u16.to_be_bytes());
        response[2..4].copy_from_slice(&70u16.to_be_bytes());
        response[4..8].copy_from_slice(&ssrc.to_be_bytes());

        let test_ip = "192.168.1.100";
        response[8..8 + test_ip.len()].copy_from_slice(test_ip.as_bytes());
        let test_port = 54321u16;
        response[72..74].copy_from_slice(&test_port.to_be_bytes());

        let (discovered_ip, discovered_port) = parse_ip_discovery_response(&response).unwrap();
        assert_eq!(discovered_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));
        assert_eq!(discovered_port, test_port);
    }
}
