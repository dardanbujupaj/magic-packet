use clap::Parser;
use magic_packet::magic::MagicPacket;
use pnet_datalink::MacAddr;
use std::error::Error;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    mac_address: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    /*
    let args = Args::parse();

    let magic_packet = MagicPacket::try_from(args.mac_address.as_str())?;

    println!("Sending magic packet for {}", args.mac_address);

    magic_packet.send()?;

    println!("done");
    */

    // magic_packet::arping::arping(MacAddr(0xb4, 0x2e, 0x99, 0x9b, 0x98, 0x5b)).unwrap();
    // magic_packet::arping::arping(MacAddr(0x80, 0xee, 0x73, 0x69, 0x78, 0x78)).unwrap();
    magic_packet::arping::arping(MacAddr(0x80, 0xee, 0x73, 0x69, 0x78, 0x78)).unwrap();

    Ok(())
}
