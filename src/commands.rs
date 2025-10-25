use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
pub enum Command {
    #[command(description = "Show help")]
    Help,
    #[command(description = "Request authorization code")]
    AuthRequest,
    #[command(description = "Authorize with access code")]
    Auth(String),
    #[command(description = "List directory contents")]
    Ls,
    #[command(description = "Change directory")]
    Cd(String),
    #[command(description = "Download file")]
    Download(String),
    #[command(description = "Execute command")]
    Exec(String),
    #[command(description = "Print working directory")]
    Pwd,
}