//! Knative Eventing ([official docs](https://knative.dev/docs/eventing/))
//!
//! # Examples
//!
//! ## Sending a single event to a broker
//! ```
//! use cloudevents::{EventBuilder, EventBuilderV10};
//! use knativity::eventing::{Client, EventSender};
//!
//! #[tokio::main]
//! async fn main() {
//!     let broker_url = std::env::var("K_BROKER").unwrap_or("http://localhost:8080".to_string());
//!     let client = Client::builder(broker_url.parse().unwrap()).build().unwrap();
//!     let ce = EventBuilderV10::new().id("my-id").ty("my-type").source("my-source").build().unwrap();
//!     client.send_event(&ce).await.unwrap();
//! }
//! ```
use cloudevents::binding::reqwest::RequestBuilderExt;
use reqwest::StatusCode;
use std::{sync::Arc, time::Duration};
use url::Url;

/// Types that implement this trait can send events to an event receiver.
#[async_trait::async_trait]
pub trait EventSender {
    /// Send a single event to the broker.
    async fn send_event<'a>(&self, event: &'a cloudevents::Event) -> Result<(), Error>;

    /// Send a batch of events to the broker.
    async fn send_event_batch<'a>(&self, events: &'a [cloudevents::Event]) -> Result<(), Error>;
}

/// Errors that can occur when sending CloudEvents to an Knative event receiver.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The request could not be delivered to the broker due to a network error.
    #[error("network error: {0}")]
    Network(Box<dyn std::error::Error + Send + 'static>),

    /// The Broker reacted with an status code that is explicitly stated as unspecified by the
    /// Knative Eventing specification.
    #[error("broker responded with unspecified HTTP status code {0}")]
    Unspecified(StatusCode),

    /// The broker failed to correctly parse the event from the request.
    #[error("broker rejected event as unparsable")]
    UnparsableEvent,

    /// The expected endpoint does not exist on the broker.
    ///
    /// # How to fix the
    /// - Check the broker url
    /// - Does the hostname resovlve correctly?
    /// - Is the targeted server a Knative Eventing broker?
    #[error("endpoint does not exist")]
    NoEndpoint,

    /// The broker returned a server side timeout.
    #[error("request timeout")]
    BrokerTimeout,

    /// A conflict occured on at the broker.
    ///
    /// The spec doesn't state in more detail what this could be.
    #[error("conflict/processing in progress")]
    Conflict,

    /// The broker is overloaded and cannot process the request.
    #[error("broker is overloaded")]
    TooManyRequests,

    /// The request could not constructed correctly.
    #[error("failed to construct request: {0}")]
    Request(Box<dyn std::error::Error + Send + 'static>),

    /// The broker responded with a status code that indicates an error which is not explicitly
    /// specified by the Knative Eventing specification.
    #[error("broker responded with a status {0}")]
    Other(StatusCode),
}

fn result_from_broker_response(response: &reqwest::Response) -> Result<(), Error> {
    let status_code = response.status();
    match status_code.as_u16() {
        200..=299 => Ok(()),
        400 => Err(Error::UnparsableEvent),
        404 => Err(Error::NoEndpoint),
        408 => Err(Error::BrokerTimeout),
        409 => Err(Error::Conflict),
        429 => Err(Error::TooManyRequests),
        s => match s {
            100..=399 => Err(Error::Unspecified(status_code)),
            400..=599 => Err(Error::Other(status_code)),
            _ => Err(Error::Unspecified(status_code)),
        },
    }
}

impl Error {
    fn is_retryable(&self) -> bool {
        match self {
            Error::NoEndpoint
            | Error::BrokerTimeout
            | Error::Conflict
            | Error::TooManyRequests
            | Error::Network(_) => true,
            _ => false,
        }
    }
}

/// A `ClientBuilder` can be used to create a `Client`.
#[derive(Clone, PartialEq, Eq)]
pub struct ClientBuilder {
    url: Url,
    request_timeout: Option<Duration>,
}

impl ClientBuilder {
    /// Contstucts a new `ClientBuilder` with the given broker url.
    pub fn new(url: Url) -> Self {
        Self {
            url,
            request_timeout: None,
        }
    }

    /// The timeout is applied per request from when the request starts connecting until the
    /// response body has finished.
    ///
    /// Default is 10 seconds.
    pub fn request_timeout(mut self, timout: Duration) -> Self {
        self.request_timeout = Some(timout);
        self
    }

    /// Returns a `Client` that uses this `ClientBuilder` configuration.
    pub fn build(self) -> Result<Client, Box<dyn std::error::Error>> {
        let http_client = reqwest::Client::builder()
            .timeout(self.request_timeout.unwrap_or(Duration::from_secs(10)))
            .build()?;
        Ok(Client(Arc::new(ClientInner {
            url: self.url,
            http_client,
        })))
    }
}

/// A client for sending CloudEvents to an Knative event receiver.
///
/// The client can be cloned and shared across threads.
pub struct Client(Arc<ClientInner>);

impl Client {
    /// Create an new client.
    pub fn builder(url: Url) -> ClientBuilder {
        ClientBuilder::new(url)
    }
}

#[async_trait::async_trait]
impl EventSender for Client {
    async fn send_event<'a>(&self, event: &'a cloudevents::Event) -> Result<(), Error> {
        self.0.send_event(event).await
    }

    async fn send_event_batch<'a>(&self, events: &'a [cloudevents::Event]) -> Result<(), Error> {
        self.0.send_event_batch(events).await
    }
}

struct ClientInner {
    url: Url,
    http_client: reqwest::Client,
}

impl ClientInner {
    async fn send_event<'a>(&self, event: &'a cloudevents::Event) -> Result<(), Error> {
        let response = self
            .http_client
            .post(self.url.clone())
            .header("Access-Control-Allow-Origin", "*")
            .event(event.clone())
            .map_err(|e| Error::Request(Box::new(e)))?
            .send()
            .await
            .map_err(|e| Error::Network(Box::new(e)))?;

        result_from_broker_response(&response)
    }

    async fn send_event_batch<'a>(&self, events: &'a [cloudevents::Event]) -> Result<(), Error> {
        let response = self
            .http_client
            .post(self.url.clone())
            .header("Access-Control-Allow-Origin", "*")
            .events(events.to_vec())
            .map_err(|e| Error::Request(Box::new(e)))?
            .send()
            .await
            .map_err(|e| Error::Network(Box::new(e)))?;

        result_from_broker_response(&response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloudevents::{EventBuilder, EventBuilderV10};
    use tracing::info;

    #[tokio::test]
    async fn test_send_event() {
        tracing_subscriber::fmt().init();
        info!("test_send_event");
        let client = ClientBuilder::new(
            "http://default-broker-ingress.default.svc.cluster.local:8080"
                .parse()
                .unwrap(),
        )
        .build()
        .unwrap();
        let ce = EventBuilderV10::new()
            .id("my-id")
            .ty("my-type")
            .source("my-source")
            .subject("its-a-subject")
            .time(chrono::Utc::now())
            .data(
                "application/json",
                cloudevents::Data::Json(serde_json::json!({"hello": "world"})),
            )
            .build()
            .unwrap();
        println!("{:?}", client.send_event(&ce).await.unwrap());
    }
}
