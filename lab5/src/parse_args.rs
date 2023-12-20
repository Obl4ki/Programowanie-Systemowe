use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short)]
    pub w: u8,

    #[arg(short)]
    pub m: u8,
}

pub fn get() -> Args {
    
    Args::parse()
}
