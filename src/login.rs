use {
    crate::Arguments,
    anyhow::{Context, Result},
    egg_mode::{auth, user},
    serde::{Deserialize, Serialize},
    std::io::Write,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    pub token: auth::Token,
    user_id: u64,
    pub username: String,
}

impl Credentials {
    pub async fn login(args: &Arguments) -> Result<Self> {
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

    pub fn load_from_file(args: &Arguments) -> Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(
            &args.creds_file,
        )?)?)
    }

    pub fn save_to_file(&self, args: &Arguments) -> Result<()> {
        Ok(std::fs::write(
            &args.creds_file,
            serde_json::to_string(self)?,
        )?)
    }

    pub fn user_id(&self) -> user::UserID {
        user::UserID::ID(self.user_id)
    }
}
