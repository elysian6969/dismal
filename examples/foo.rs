use dismal::Inst;

fn test(bytes: &[u8]) {
    println!("---");
    println!("bytes = {bytes:02X?}");

    if let Some(inst) = Inst::from_bytes(bytes) {
        println!("inst = {inst:0X?}");
        println!("reenc = {:02X?}", inst.to_bytes());
    } else {
        println!("failed to decode");
    }

    println!("---");
}

fn main() {
    test(&[0x48, 0x8B, 0x05, 0x5C, 0x4D, 0x99, 0x01]);

    test(&[0xE8, 0x70, 0xB2, 0x5E, 0x00]);

    test(&[0x50]);

    test(&[0xFF, 0x15, 0x69, 0x69, 0x69, 0x69]);

    test(&[0x58]);

    test(&[0xC3]);

    test(&[0x41, 0x5F]);
}
