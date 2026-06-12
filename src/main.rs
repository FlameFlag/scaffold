mod cli;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{:?}", miette::Report::new(err));
        std::process::exit(1);
    }
}
