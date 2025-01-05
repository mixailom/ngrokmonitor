use anyhow::{Context, Result};
use lettre::{message::Mailbox, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use reqwest::{self, Client};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;
extern crate ini;
use ini::Ini;
use std::{thread, time};
use log::{info, error};

#[derive(Debug, Deserialize)]
struct NgrokTunnels {
    tunnels: Vec<Tunnel>,
}

#[derive(Debug, Deserialize)]
struct Tunnel {
    name: String,
    public_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // Load configuration from the file
    let config_path = "/etc/ngrokmonitor/ngrokmonitor.cfg";
    let config = Ini::load_from_file(config_path).context(format!("Failed to load configuration file from {}", config_path))?;
    let smtp_section = config.section(Some("smtp")).unwrap();

    let smtp_server= smtp_section.get("server").unwrap_or("smtp.gmail.com").to_string();
    let smtp_port: u16 = smtp_section
        .get( "port")
        .unwrap_or("587")
        .parse()
        .context("Invalid SMTP port")?;
    let smtp_username = smtp_section.get("username").unwrap_or("").to_string();
    let smtp_password = smtp_section.get("password").unwrap_or("").to_string();
    let email_section = config.section(Some("email")).unwrap();
    let recipient = email_section.get("recipient").unwrap_or("").to_string();
    let subject = email_section
        .get("subject")
        .unwrap_or("Ngrok Tunnel Changed")
        .to_string();
    let body_template = email_section
        .get("body_template")
        .unwrap_or("The ngrok tunnel public URL has changed. New URL: {url}")
        .to_string();
    let interval_seconds: u64 = email_section
        .get("interval_seconds")
        .unwrap_or("10")
        .parse()
        .context("Invalid monitor interval")?;

    let mut last_tunnel_url = String::new(); // Keep track of the last known TCP tunnel URL
    let client = Client::new();
    let url = "http://127.0.0.1:4040/api/tunnels";

    loop {
         match fetch_ngrok_tunnel(&client, url).await {
            Ok(current_tunnel_url) => {
                info!("Current ngrok tunnel URL: {}", current_tunnel_url);
                if current_tunnel_url != last_tunnel_url {
                    info!("TCP Tunnel Address changed: {}", current_tunnel_url);
                    let body = body_template.replace("{url}", &current_tunnel_url);
                    send_email(
                        &smtp_server,
                        smtp_port,
                        &smtp_username,
                        &smtp_password,
                        &recipient,
                        &subject,
                        &body,
                    )?;
                    // Update the last known tunnel URL
                    last_tunnel_url = current_tunnel_url.clone();
                }
            }
            Err(e) => {
                // Log the error but continue the loop
                error!("Error fetching ngrok tunnel: {}", e);
            }
        }

        // Sleep for the specified interval before the next check
        thread::sleep(time::Duration::from_secs(interval_seconds));
    }
}


async fn fetch_ngrok_tunnel(client: &Client, url: &str) -> Result<String> {   
        // Await the response and handle errors
        let response = client.get(url).send().await.context("Failed to send request")?;
    
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch ngrok tunnels: {}", response.status()).into());
        }
    
        let json: serde_json::Value = response.json().await.context("Failed to parse JSON response")?;
        let tunnel_url = json["tunnels"][0]["public_url"]
            .as_str()
            .context("Tunnel URL not found in the response")?;
    
        Ok(tunnel_url.to_string()) // Return the URL as a String
    }

/// Loads the configuration file
fn load_config(file_path: &str) -> Result<Ini> {
    let config = Ini::load_from_file(file_path).context("Failed to read configuration file")?;
    Ok(config)
}

/// Sends an email notification
fn send_email(
    smtp_server: &str,
    smtp_port: u16,
    smtp_user: &str,
    smtp_password: &str,
    recipient: &str,
    subject: &str,
    body: &str,
) -> Result<()> {
    let email = Message::builder()
        .from(Mailbox::new(Some("Ngrok Monitor".into()), smtp_user.parse()?))
        .to(Mailbox::new(None, recipient.parse()?))
        .subject(subject)
        .body(body.to_string())?;

    let creds = Credentials::new(smtp_user.to_string(), smtp_password.to_string());
    let mailer = match SmtpTransport::relay(smtp_server) {
        Ok(mailer) => mailer.port(smtp_port).credentials(creds).build(),
        Err(e) => {
            error!("Failed to build mailer: {}", e);
            return Err(anyhow::anyhow!("Failed to build mailer").into());
        }
    };

    match mailer.send(&email) {
        Ok(_) => {
            info!("Email sent successfully!");
            Ok(())
        }
        Err(e) => {
            error!("Failed to send email: {}", e);
            //Err(anyhow::anyhow!("Failed to send email").into())
            Ok(())
        }
    }
}
