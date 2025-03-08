use std::io::{stdin, Read};

use clap::Parser;
use mdsvexrs::Context;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    layout: String,
    // #[arg(short, long)]
    // path: String,
    #[arg(long)]
    timings: bool,
}

fn main() {
    let args = Args::parse();
    let mut ctx = Context::new(mdsvexrs::MdsvexrsOptions {
        layout: args.layout,
        // path: args.path,
    });

    let mut input = String::new();
    stdin().read_to_string(&mut input).unwrap();
    let output = ctx.convert(&input);

    if args.timings {
        ctx.print_timings();
        return;
    }

    print!("{output}");
}
