use clap::Clap;

mod dhcp;
/// run the rustboot server
#[derive(Clap)]
#[clap(version = "1.0", author = "Adam Young <adam@younglogic.com>")]
struct Opts {
    /// Sets a custom config file. 
    #[clap(short, long, default_value = "default.conf")]
    config: String,

    /// Directory where to write captured packets
    #[clap(short, long, default_value = "/tmp/rustboot/")]
    packet_capture_dir: String,


    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    /// If the server should write captured packets to disk
    #[clap(short)]
    write_capture: bool,
}

fn main() -> std::io::Result<()> {

    let opts: Opts = Opts::parse();
    println!("Value for config: {}", opts.config);
    let server = dhcp::DHCPServer::new( opts.verbose > 0,
                                        opts.write_capture,
                                        &opts.packet_capture_dir )?;
    server.run()?;
    Ok(())
}
