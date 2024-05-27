use crate::config::Config;
use crate::headers::ContentType;
use crate::request::Request;
use crate::response::Response;

pub struct Context {
    pub config: Config,
    pub request: Request,
    pub response: Response,
}

impl Context {
    pub fn new(config: Config, request: Request, response: Response) -> Self {
        Context {
            config,
            request,
            response,
        }
    }

    pub fn send_file(&mut self, path: &str) {
        println!("[context] sending file: {}", path);
        self.response.content_type(ContentType::HTML);
        self.response.file(path);
    }
}
