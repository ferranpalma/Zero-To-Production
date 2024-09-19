use reqwest::{Client, Url};
use secrecy::{ExposeSecret, SecretString};

use crate::models::SubscriberEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: Url,
    sender: SubscriberEmail,
    api_token: SecretString,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct EmailRequestBody<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

impl EmailClient {
    pub fn new(
        base_url: &str,
        sender: SubscriberEmail,
        api_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(timeout)
                .build()
                .expect("Failed to build HTTP client for email client."),
            base_url: Url::parse(base_url).expect("Failed to parse email client's base url"),
            sender,
            api_token,
        }
    }
    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = self
            .base_url
            .join("/email")
            .expect("Failed to parse email client's url");
        let body = EmailRequestBody {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };

        self.http_client
            .post(url)
            .header("X-Postmark-Server-Token", self.api_token.expose_secret())
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake, Faker,
    };
    use secrecy::Secret;
    use wiremock::{
        matchers::{any, header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::EmailClient;
    use crate::models::SubscriberEmail;

    fn get_email_subject() -> String {
        Sentence(1..10).fake()
    }
    fn get_email_content() -> String {
        Paragraph(1..10).fake()
    }
    fn get_email_address() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }
    fn get_email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            &base_url,
            get_email_address(),
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }

    struct EmailRequestBodyMatcher;

    impl wiremock::Match for EmailRequestBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn test_email_client_fires_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(EmailRequestBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let _ = email_client
            .send_email(
                &get_email_address(),
                &get_email_subject(),
                &get_email_content(),
                &get_email_content(),
            )
            .await;
    }

    #[tokio::test]
    async fn test_email_client_send_function_returns_ok_if_response_is_200() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let send_email_result = email_client
            .send_email(
                &get_email_address(),
                &get_email_subject(),
                &get_email_content(),
                &get_email_content(),
            )
            .await;

        assert_ok!(send_email_result);
    }

    #[tokio::test]
    async fn test_email_client_send_function_returns_err_if_response_is_500() {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let send_email_result = email_client
            .send_email(
                &get_email_address(),
                &get_email_subject(),
                &get_email_content(),
                &get_email_content(),
            )
            .await;

        assert_err!(send_email_result);
    }

    #[tokio::test]
    async fn test_email_client_send_function_returns_err_if_server_takes_more_than_10s_to_respond()
    {
        let mock_server = MockServer::start().await;
        let email_client = get_email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(20)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let send_email_result = email_client
            .send_email(
                &get_email_address(),
                &get_email_subject(),
                &get_email_content(),
                &get_email_content(),
            )
            .await;

        assert_err!(send_email_result);
    }
}
