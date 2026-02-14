//! VISCA command encoding for Sony and compatible PTZ cameras.
//! VISCA-over-IP uses UDP with a framing header.

/// VISCA-over-IP framing header (8 bytes).
pub struct ViscaIpHeader {
    pub payload_type: u16,
    pub payload_length: u16,
    pub sequence_number: u32,
}

impl ViscaIpHeader {
    pub const COMMAND: u16 = 0x0100;
    pub const INQUIRY: u16 = 0x0110;

    pub fn new_command(payload_length: u16, seq: u32) -> Self {
        Self {
            payload_type: Self::COMMAND,
            payload_length,
            sequence_number: seq,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8);
        buf.extend_from_slice(&self.payload_type.to_be_bytes());
        buf.extend_from_slice(&self.payload_length.to_be_bytes());
        buf.extend_from_slice(&self.sequence_number.to_be_bytes());
        buf
    }
}

/// Build a full VISCA-over-IP packet (header + payload).
pub fn build_visca_packet(payload: &[u8], seq: u32) -> Vec<u8> {
    let header = ViscaIpHeader::new_command(payload.len() as u16, seq);
    let mut packet = header.to_bytes();
    packet.extend_from_slice(payload);
    packet
}

/// VISCA absolute pan/tilt position command.
/// pan: 16-bit signed, range 0xFC90 to 0x0370
/// tilt: 16-bit signed, range 0xFE70 to 0x0120
/// speed: 1-24 for pan, 1-23 for tilt
pub fn pan_tilt_absolute(pan_speed: u8, tilt_speed: u8, pan: i16, tilt: i16) -> Vec<u8> {
    let pan_bytes = (pan as u16).to_be_bytes();
    let tilt_bytes = (tilt as u16).to_be_bytes();
    vec![
        0x81,
        0x01,
        0x06,
        0x02,
        pan_speed,
        tilt_speed,
        // Pan position (4 nibbles)
        (pan_bytes[0] >> 4) & 0x0F,
        pan_bytes[0] & 0x0F,
        (pan_bytes[1] >> 4) & 0x0F,
        pan_bytes[1] & 0x0F,
        // Tilt position (4 nibbles)
        (tilt_bytes[0] >> 4) & 0x0F,
        tilt_bytes[0] & 0x0F,
        (tilt_bytes[1] >> 4) & 0x0F,
        tilt_bytes[1] & 0x0F,
        0xFF,
    ]
}

/// VISCA relative pan/tilt movement.
/// direction: pan_speed (01-18h), tilt_speed (01-17h)
/// 01=up, 02=down, 03=stop for tilt
/// 01=left, 02=right, 03=stop for pan
pub fn pan_tilt_relative(pan_speed: u8, tilt_speed: u8, pan_dir: u8, tilt_dir: u8) -> Vec<u8> {
    vec![
        0x81, 0x01, 0x06, 0x01, pan_speed, tilt_speed, pan_dir, tilt_dir, 0xFF,
    ]
}

/// VISCA pan/tilt stop.
pub fn pan_tilt_stop() -> Vec<u8> {
    vec![0x81, 0x01, 0x06, 0x01, 0x00, 0x00, 0x03, 0x03, 0xFF]
}

/// VISCA zoom absolute position (0x0000 to 0x4000).
pub fn zoom_absolute(position: u16) -> Vec<u8> {
    let bytes = position.to_be_bytes();
    vec![
        0x81,
        0x01,
        0x04,
        0x47,
        (bytes[0] >> 4) & 0x0F,
        bytes[0] & 0x0F,
        (bytes[1] >> 4) & 0x0F,
        bytes[1] & 0x0F,
        0xFF,
    ]
}

/// VISCA preset recall: 81 01 04 3F 02 pp FF
pub fn preset_recall(preset_number: u8) -> Vec<u8> {
    vec![0x81, 0x01, 0x04, 0x3F, 0x02, preset_number, 0xFF]
}

/// VISCA preset store: 81 01 04 3F 01 pp FF
pub fn preset_store(preset_number: u8) -> Vec<u8> {
    vec![0x81, 0x01, 0x04, 0x3F, 0x01, preset_number, 0xFF]
}

/// VISCA position inquiry command.
pub fn pan_tilt_position_inquiry() -> Vec<u8> {
    vec![0x81, 0x09, 0x06, 0x12, 0xFF]
}

/// VISCA zoom position inquiry command.
pub fn zoom_position_inquiry() -> Vec<u8> {
    vec![0x81, 0x09, 0x04, 0x47, 0xFF]
}

/// Convert normalized pan (-1.0 to 1.0) to VISCA pan value.
/// VISCA range: 0xFC90 (-880) to 0x0370 (880)
pub fn normalize_to_visca_pan(normalized: f64) -> i16 {
    let clamped = normalized.clamp(-1.0, 1.0);
    (clamped * 880.0) as i16
}

/// Convert normalized tilt (-1.0 to 1.0) to VISCA tilt value.
/// VISCA range: 0xFE70 (-400) to 0x0120 (288)
pub fn normalize_to_visca_tilt(normalized: f64) -> i16 {
    let clamped = normalized.clamp(-1.0, 1.0);
    // Map -1..1 to -400..288 (asymmetric range centered approximately)
    let center = (-400.0 + 288.0) / 2.0; // -56
    let half_range = (288.0 - (-400.0)) / 2.0; // 344
    (center + clamped * half_range) as i16
}

/// Convert normalized zoom (0.0 to 1.0) to VISCA zoom value.
/// VISCA range: 0x0000 to 0x4000
pub fn normalize_to_visca_zoom(normalized: f64) -> u16 {
    let clamped = normalized.clamp(0.0, 1.0);
    (clamped * 0x4000 as f64) as u16
}
