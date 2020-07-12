use {
    anyhow::{Context, Result},
    egg_mode::{auth, tweet, user},
    serde::{Deserialize, Serialize},
    std::io::Write,
    structopt::StructOpt,
};

#[derive(StructOpt, Debug)]
#[structopt(about = "Twitter likes scraper")]
struct Arguments {
    /// Twitter API key. Loaded from $CWD/api_keys.env by default.
    #[structopt(long, env = "API_KEY", hide_env_values = true)]
    api_key: String,

    /// Twitter API secret. Loaded from $CWD/api_keys.env by default.
    #[structopt(long, env = "API_SECRET", hide_env_values = true)]
    api_secret: String,

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
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::from_filename("api_keys.env").ok();
    let args = Arguments::from_args();

    let creds = if let Command::Login = args.cmd {
        let creds = Credentials::login(&args).await.context("Login failed")?;
        creds
            .save_to_file(&args)
            .context("Failed to save user credentials")?;
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
    }

    Ok(())
}
#[derive(Serialize, Deserialize, Debug)]
struct Credentials {
    token: auth::Token,
    user_id: u64,
    username: String,
}

impl Credentials {
    async fn login(args: &Arguments) -> Result<Self> {
        let con_token = auth::KeyPair::new(args.api_key.clone(), args.api_secret.clone());
        let request_token = auth::request_token(&con_token, "oob").await?;
        let auth_url = auth::authorize_url(&request_token);

        println!("Visit the following URL, login, and paste the PIN provided below.\n");
        println!("URL: {}\n", auth_url);
        print!("PIN: ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("Failed to read PIN from stdin")?;

        let (token, user_id, username) = auth::access_token(con_token, &request_token, input)
            .await
            .context("Failed to get access token")?;

        Ok(Self {
            token,
            user_id,
            username,
        })
    }

    fn load_from_file(args: &Arguments) -> Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(
            &args.creds_file,
        )?)?)
    }

    fn save_to_file(&self, args: &Arguments) -> Result<()> {
        Ok(std::fs::write(
            &args.creds_file,
            serde_json::to_string(self)?,
        )?)
    }

    fn user_id(&self) -> user::UserID {
        user::UserID::ID(self.user_id)
    }
}
