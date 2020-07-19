mod login;
mod migrations;

use {
    crate::login::Credentials,
    anyhow::{Context, Result},
    diesel::prelude::*,
    diesel_migrations::run_migrations,
    egg_mode::tweet,
    structopt::StructOpt,
};

#[derive(StructOpt, Debug)]
#[structopt(about = "Twitter likes scraper")]
pub struct Arguments {
    /// Twitter API key. Loaded from $CWD/.env by default.
    #[structopt(long, env = "API_KEY", hide_env_values = true)]
    api_key: String,

    /// Twitter API secret. Loaded from $CWD/.env by default.
    #[structopt(long, env = "API_SECRET", hide_env_values = true)]
    api_secret: String,

    /// Sqlite database URL. Loaded from $CWD/.env by default.
    #[structopt(long, env = "DATABASE_URL", hide_env_values = true)]
    database_url: String,

    /// User credentials file.
    // TODO: Saving user credentials to a plain JSON file is a bodge; use a more secure store.
    #[structopt(long, default_value = "user_creds.json")]
    creds_file: String,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Login to Twitter.
    Login,

    /// Placeholder to test login.
    PrintLikes,

    /// Placeholder to test migrations.
    Migrations,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let args = Arguments::from_args();

    let creds = if let Command::Login = args.cmd {
        let creds = Credentials::login(&args).await.context("Login failed")?;
        creds.save_to_file(&args).context("Failed to save user credentials")?;
        creds
    } else {
        Credentials::load_from_file(&args)
            .context("Failed to load user credentials; do you need to login?")?
    };
    println!("User {} logged in", creds.username);

    match args.cmd {
        Command::Login => {}
        Command::PrintLikes => {
            let timeline = tweet::liked_by(creds.user_id(), &creds.token);
            let (_timeline, tweets) = timeline.start().await.context("Failed to get likes")?;
            println!("{:#?}", tweets);
        }
        Command::Migrations => {
            // Run any pending migrations.
            let conn = SqliteConnection::establish(&args.database_url)
                .context("Error connecting to database")?;
            let migrations = migrations::embedded_migrations(&conn);
            run_migrations(&conn, migrations, &mut std::io::stdout())
                .context("Error running database migrations")?;
        }
    }
    Ok(())
}
