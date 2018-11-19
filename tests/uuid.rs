use bluster::SdpShortUuid;
use uuid::Uuid;

#[test]
fn test_from_sdp_short_uuid() {
    Uuid::from_sdp_short_uuid(0x0000 as u16);
    Uuid::from_sdp_short_uuid(0x0000 as u32);
}
