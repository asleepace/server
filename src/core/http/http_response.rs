pub struct HttpResponse {
    pub version: String,
    pub status: String,
    pub code: u16,
    data: String,
}

static CRLF: &str = "\r\n";

fn with_crlf(data: &str) -> String {
    format!("{}{}", data, CRLF)
}

impl HttpResponse {
    pub fn new() -> Self {
        HttpResponse {
            version: String::from("HTTP/1.1"),
            status: String::from("OK"),
            data: String::new(),
            code: 200,
        }
    }

    pub fn append(&mut self, data: &str) -> &mut Self {
        self.data.push_str(with_crlf(data).as_str());
        self
    }

    fn http_headers(&self) -> String {
        let headers = format!("{} {} {}", self.version, self.code, self.status);
        with_crlf(headers.as_str())
    }

    pub fn append_body(&self, body: &mut Vec<u8>) -> Vec<u8> {
        let mut response = String::new();
        let http_headers = self.http_headers();
        response.push_str(http_headers.as_str());
        response.push_str(CRLF);
        response.push_str(CRLF);
        let mut response_bytes = response.into_bytes();
        response_bytes.append(body);
        response_bytes
    }
}

/**
    Create a new HTTP response which can be used to send data back to the client.
    Below is an example of how to use this function:
    ```
    use http::http_response::create_http_response;
    let response = create_http_response();
    response.append("Content-Type: text/html");
    ```
*/
pub fn create_http_response() -> HttpResponse {
    let mut response = HttpResponse::new();
    response
        .append("Content-Type: */*")
        .append("<h1>Hello, World!</h1>");

    response
}
