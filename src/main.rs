use std::io::{stdin, Read};

use clap::{Parser, Subcommand};
use mdsvexrs::Context;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    layout: String,
    #[arg(short, long, default_value = "")]
    custom_tags: Vec<String>,
    // #[arg(short, long)]
    // path: String,
    #[arg(long)]
    timings: bool,
}


fn main() {
    let args = Args::parse();

    let mut ctx = Context::new(mdsvexrs::MdsvexrsOptions {
        layout: args.layout,
        custom_tags: args.custom_tags,
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
