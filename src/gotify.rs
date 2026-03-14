use serde::Serialize;

#[derive(Serialize)]
struct GotifyMessage {
    title: String,
    message: String,
    priority: u8,
}

pub struct GotifyClient {
    url: String,
    token: String,
}

impl GotifyClient {
    pub fn new(url: String, token: String) -> Self {
        Self { url, token }
    }

    pub fn send(
        &self,
        title: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg = GotifyMessage {
            title: title.to_string(),
            message: message.to_string(),
            priority: 5,
        };

        let _response = ureq::post(&format!("{}/message", self.url))
            .set("X-Gotify-Key", &self.token)
            .send_json(&msg)?;

        Ok(())
    }
}

pub fn notify_task_complete(task_name: &str, details: &str) {
    let gotify = match (std::env::var("GOTIFY_URL"), std::env::var("GOTIFY_TOKEN")) {
        (Ok(url), Ok(token)) => Some(GotifyClient::new(url, token)),
        _ => None,
    };

    if let Some(client) = gotify {
        let title = format!("[RustMUD] 任务完成: {}", task_name);
        let message = format!(
            "{}\n\n时间: {}",
            details,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        if let Err(e) = client.send(&title, &message) {
            tracing::warn!("Failed to send Gotify notification: {}", e);
        }
    }
}
