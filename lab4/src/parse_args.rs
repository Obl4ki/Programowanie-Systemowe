// https://docs.rs/clap/latest/clap/
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, default_value_t = false)]
    pub verbose: bool,

    #[arg(short, default_value_t = 1)]
    pub times: u8,

    #[arg(short)]
    pub command: String,
}

pub fn get() -> Args {
    let args = Args::parse();
    return args;
}
