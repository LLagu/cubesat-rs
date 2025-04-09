use spacepacket::{GroupingFlag, PacketType, SpacePacket};
fn main() {
    let payload = b"secret payload".to_vec();
    let packet = SpacePacket::new(
        0,
        PacketType::Command,
        0x012,
        GroupingFlag::Unsegm,
        3,
        true,
        payload,
    );
    
    let bytestream = packet.encode();
    let recovered_packet = SpacePacket::decode(&mut bytestream.as_slice()).unwrap();
    assert_eq!(packet, recovered_packet)
}
