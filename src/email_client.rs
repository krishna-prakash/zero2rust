use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

use crate::domain::SubscriberEmail;

#[derive(Clone)]
pub struct EmailClient {
    auth_token: Secret<String>,
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
}

#[derive(Serialize)]
pub struct EmailDetails<'a> {
    name: &'a str,
    email: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendEmailRequest<'a> {
    sender: EmailDetails<'a>,
    to: Vec<EmailDetails<'a>>,
    subject: &'a str,
    html_content: &'a str,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        auth_token: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();

        Self {
            http_client,
            base_url,
            sender,
            auth_token,
        }
    }
    pub async fn send_email(
        &self,
        recepient: SubscriberEmail,
        subject: &str,
        html_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let req_body = SendEmailRequest {
            sender: EmailDetails {
                name: "test",
                email: self.sender.as_ref(),
            },
            to: vec![EmailDetails {
                name: "test",
                email: recepient.as_ref(),
            }],
            subject,
            html_content,
        };
        self.http_client
            .post(&url)
            .header("api-key", self.auth_token.expose_secret())
            .json(&req_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::domain::SubscriberEmail;

    use super::EmailClient;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("sender").is_some()
                    && body.get("to").is_some()
                    && body.get("subject").is_some()
                    && body.get("htmlContent").is_some()
            } else {
                false
            }
        }
    }

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }

    #[actix_web::test]
    async fn send_email_fires_a_req_with_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("api-key"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content())
            .await;

        assert_ok!(outcome);
    }

    #[actix_web::test]
    async fn send_email_fails_if_500_is_retruned() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("api-key"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content())
            .await;

        assert_err!(outcome);
    }

    #[actix_web::test]
    async fn send_email_fails_if_timedout() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));

        Mock::given(header_exists("api-key"))
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content())
            .await;

        assert_err!(outcome);
    }
}
