use reqwest::Client;
use std::time::Duration;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum RemoteIcsError {
    #[error("invalid url")]
    InvalidUrl,
    #[error("remote request failed: {0}")]
    FetchFailed(String),
    #[error("remote request timed out")]
    Timeout,
    #[error("remote calendar is too large")]
    TooLarge,
}

pub async fn fetch_remote_ics(
    url: &str,
    timeout: Duration,
    max_bytes: usize,
) -> Result<String, RemoteIcsError> {
    let parsed = Url::parse(url).map_err(|_| RemoteIcsError::InvalidUrl)?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(RemoteIcsError::InvalidUrl);
    }

    let client = Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|err| RemoteIcsError::FetchFailed(err.to_string()))?;
    let response = client
        .get(parsed)
        .send()
        .await
        .map_err(|err| {
            if err.is_timeout() {
                RemoteIcsError::Timeout
            } else {
                RemoteIcsError::FetchFailed(err.to_string())
            }
        })?
        .error_for_status()
        .map_err(|err| RemoteIcsError::FetchFailed(err.to_string()))?;
    let bytes = response.bytes().await.map_err(|err| {
        if err.is_timeout() {
            RemoteIcsError::Timeout
        } else {
            RemoteIcsError::FetchFailed(err.to_string())
        }
    })?;
    if bytes.len() > max_bytes {
        return Err(RemoteIcsError::TooLarge);
    }
    String::from_utf8(bytes.to_vec()).map_err(|err| RemoteIcsError::FetchFailed(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    const ICS: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:remote-1\r\nSUMMARY:Remote\r\nDTSTART:20260601T090000Z\r\nDTEND:20260601T100000Z\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

    #[tokio::test]
    async fn fetches_remote_ics_from_http_url() {
        let url = spawn_http_server(200, ICS);

        let body = fetch_remote_ics(&url, Duration::from_secs(2), 20_000)
            .await
            .expect("remote fixture should fetch");

        assert!(body.contains("BEGIN:VCALENDAR"));
        assert!(body.contains("UID:remote-1"));
    }

    #[tokio::test]
    async fn rejects_non_http_protocols() {
        let error = fetch_remote_ics("file:///tmp/calendar.ics", Duration::from_secs(2), 20_000)
            .await
            .expect_err("file protocol must be rejected");

        assert!(matches!(error, RemoteIcsError::InvalidUrl));
    }

    #[tokio::test]
    async fn rejects_oversized_remote_ics_response() {
        let url = spawn_http_server(200, ICS);

        let error = fetch_remote_ics(&url, Duration::from_secs(2), 10)
            .await
            .expect_err("oversized response must be rejected");

        assert!(matches!(error, RemoteIcsError::TooLarge));
    }

    #[tokio::test]
    async fn maps_http_errors_to_fetch_failed() {
        let url = spawn_http_server(404, "missing");

        let error = fetch_remote_ics(&url, Duration::from_secs(2), 20_000)
            .await
            .expect_err("404 should fail");

        assert!(matches!(error, RemoteIcsError::FetchFailed(_)));
    }

    fn spawn_http_server(status: u16, body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("read local addr");
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                drain_request(&mut stream);
                let reason = if status == 200 { "OK" } else { "Not Found" };
                let response = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: text/calendar\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        format!("http://{address}/calendar.ics")
    }

    fn drain_request(stream: &mut TcpStream) {
        let mut buffer = [0_u8; 1024];
        let _ = stream.read(&mut buffer);
    }
}
