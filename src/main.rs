use std::error::Error;

use clap::Parser;
use magic_packet::MagicPacket;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    mac_address: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let magic_packet = MagicPacket::try_from(args.mac_address.as_str())?;

    println!("Sending magic packet for {}", args.mac_address);

    magic_packet.send()?;

    println!("done");

    Ok(())
}
