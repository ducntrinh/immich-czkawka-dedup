use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(long, default_value = "http://localhost:2283")]
    pub immich_host: String,

    #[clap(long)]
    pub immich_api_key: String,

    #[clap(long)]
    pub czkawka_output_path: String,

    #[clap(long)]
    pub spare_similarity: bool,

    #[clap(long)]
    pub dry_run: bool,

    #[clap(long, default_value = "64")]
    pub concurrency: usize,
}
