//! `cleansys-web` — local web UI.
//!
//! Binds `127.0.0.1:3000` by default; `--host` / `--port` (or the `HOST` / `PORT`
//! env vars) override. Opens your browser on start; pass `--no-open` to skip.

use clap::Parser;
use cleansys_web::{ServerOptions, serve};

#[derive(Debug, Parser)]
#[command(name = "cleansys-web", version, about)]
struct Args {
    /// Interface to bind (0.0.0.0 exposes it on the network — there is no auth).
    #[arg(long, env = "HOST", default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on.
    #[arg(long, short, env = "PORT", default_value_t = 3000)]
    port: u16,

    /// Don't open the UI in a browser on start.
    #[arg(long)]
    no_open: bool,

    /// Print every theme name and exit.
    #[arg(long)]
    list_themes: bool,

    /// Colour theme (remembered for the GUI too); see --list-themes.
    #[arg(long, value_name = "NAME")]
    theme: Option<String>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    if args.list_themes {
        for n in cleansys_web::theme::names() {
            println!("{n}");
        }
        return Ok(());
    }
    if let Some(name) = args.theme {
        match cleansys_web::theme::names()
            .iter()
            .position(|n| n.eq_ignore_ascii_case(&name))
        {
            Some(i) => cleansys_web::theme::save_index(i),
            None => {
                eprintln!("unknown theme '{name}' (try --list-themes)");
                std::process::exit(2);
            }
        }
    }
    serve(ServerOptions {
        host: args.host,
        port: args.port,
        open_browser: !args.no_open,
    })
    .await
}
