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
    test(&[0x50]);

    test(&[0xFF, 0x15, 0x69, 0x69, 0x69, 0x69]);

    test(&[0x58]);

    test(&[0xC3]);

    test(&[0x41, 0x5F]);
}