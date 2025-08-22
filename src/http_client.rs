//! Wrapper to make reqwest_middleware::ClientWithMiddleware work with async-openai's HttpClient trait

use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use reqwest::{header::HeaderMap, Method, Url};
use reqwest_middleware::ClientWithMiddleware;
use std::pin::Pin;

/// Wrapper struct for ClientWithMiddleware to implement HttpClient
pub struct HttpClientWithMiddleware {
    client: ClientWithMiddleware,
}

impl HttpClientWithMiddleware {
    pub fn new(client: ClientWithMiddleware) -> Self {
        Self { client }
    }
}

/// Implementation of async-openai's HttpClient trait for our wrapper
#[async_trait]
impl async_openai::http_client::HttpClient for HttpClientWithMiddleware {
    async fn request(
        &self,
        method: Method,
        url: Url,
        headers: HeaderMap,
        body: Option<Bytes>,
    ) -> Result<async_openai::http_client::HttpResponse, async_openai::http_client::HttpError> {
        let mut request = self.client.request(method, url).headers(headers);

        if let Some(body) = body {
            request = request.body(body);
        }

        let response = request
            .send()
            .await
            .map_err(|e| async_openai::http_client::HttpError {
                message: e.to_string(),
                status: match &e {
                    reqwest_middleware::Error::Reqwest(re) => re.status(),
                    _ => None,
                },
            })?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .bytes()
            .await
            .map_err(|e| async_openai::http_client::HttpError {
                message: e.to_string(),
                status: Some(status),
            })?;

        Ok(async_openai::http_client::HttpResponse {
            status,
            headers,
            body,
        })
    }

    async fn request_multipart(
        &self,
        method: Method,
        url: Url,
        mut headers: HeaderMap,
        form: async_openai::http_client::MultipartForm,
    ) -> Result<async_openai::http_client::HttpResponse, async_openai::http_client::HttpError> {
        // The MultipartForm already contains the encoded body and boundary
        // We need to set the Content-Type header with the boundary
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={}", form.boundary)
                .parse()
                .map_err(|e: reqwest::header::InvalidHeaderValue| {
                    async_openai::http_client::HttpError {
                        message: e.to_string(),
                        status: None,
                    }
                })?,
        );

        let response = self
            .client
            .request(method, url)
            .headers(headers)
            .body(form.body)
            .send()
            .await
            .map_err(|e| async_openai::http_client::HttpError {
                message: e.to_string(),
                status: match &e {
                    reqwest_middleware::Error::Reqwest(re) => re.status(),
                    _ => None,
                },
            })?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .bytes()
            .await
            .map_err(|e| async_openai::http_client::HttpError {
                message: e.to_string(),
                status: Some(status),
            })?;

        Ok(async_openai::http_client::HttpResponse {
            status,
            headers,
            body,
        })
    }

    async fn request_stream(
        &self,
        method: Method,
        url: Url,
        headers: HeaderMap,
        body: Option<Bytes>,
    ) -> Result<
        Pin<
            Box<
                dyn Stream<
                        Item = Result<
                            async_openai::http_client::SseEvent,
                            async_openai::http_client::HttpError,
                        >,
                    > + Send,
            >,
        >,
        async_openai::http_client::HttpError,
    > {
        use eventsource_stream::Eventsource;
        use futures::StreamExt;

        let mut request = self.client.request(method, url).headers(headers);

        if let Some(body) = body {
            request = request.body(body);
        }

        let response = request
            .send()
            .await
            .map_err(|e| async_openai::http_client::HttpError {
                message: e.to_string(),
                status: match &e {
                    reqwest_middleware::Error::Reqwest(re) => re.status(),
                    _ => None,
                },
            })?;

        let stream = response.bytes_stream().eventsource();

        // Convert eventsource Event to our SseEvent
        let converted_stream = stream.map(move |event| {
            match event {
                Ok(event) => Ok(async_openai::http_client::SseEvent {
                    data: event.data,
                    event: if event.event.is_empty() {
                        None
                    } else {
                        Some(event.event)
                    },
                    id: if event.id.is_empty() {
                        None
                    } else {
                        Some(event.id)
                    },
                    retry: None, // eventsource_stream doesn't expose retry
                }),
                Err(e) => Err(async_openai::http_client::HttpError {
                    message: e.to_string(),
                    status: None,
                }),
            }
        });

        Ok(Box::pin(converted_stream))
    }
}
