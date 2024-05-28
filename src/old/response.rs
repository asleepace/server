// use crate::headers::{ContentType, Headers};
// use std::{
//     fs,
//     io::{Error, Write},
//     net::TcpStream,
//     result::Result,
// };

// pub struct Response {
//     headers: Headers,
//     body: String,
//     status: u16,
// }

// pub fn create_response() -> String {
//     let mut response = String::new();
//     response.push_str("HTTP/1.1 200 OK\r\n");
//     response
// }

// pub fn send_reponse(tcp_stream: &TcpStream) -> Result<bool, ()> {
//     // STEP 1: Write headers
//     let mut response = create_response();
//     response.push_str("Content-Type: text/html; charset=utf-8\r\n");

//     // STEP 2: Write body

//     // STEP 3: Write to stream

//     Ok(true)
// }

// impl Response {
//     pub fn new(stream: &TcpStream) -> Self {
//         Response {
//             headers: Headers::new(),
//             body: String::new(),
//             status: 200,
//             stream,
//         }
//     }

//     pub fn content_type(&mut self, content_type: ContentType) {
//         self.headers.set_content_type(content_type);
//     }

//     pub fn status(&mut self, status: u16) {
//         self.status = status;
//     }

//     pub fn header(&mut self, header: &str, value: &str) {
//         self.headers.set(header.to_string(), value.to_string());
//     }

//     pub fn body(&mut self, body: String) {
//         self.headers.set_content_length(body.len());
//         self.body = body;
//     }

//     pub fn file(&mut self, file: &str) -> Result<(), ()> {
//         let mut formatted_file = file.trim_matches('/');
//         formatted_file = formatted_file.trim_matches('.');
//         let file_path = format!("./src/public/{}", formatted_file);

//         // let mut writer = BufWriter::new(&self.stream);

//         let bytes = match fs::read(file_path.clone()) {
//             Ok(data) => data,
//             Err(_) => {
//                 eprintln!("[response] file not found: {}", file_path);
//                 self.status(404);
//                 return Result::Err(());
//             }
//         };

//         let total_bytes_to_send = bytes.len();
//         println!("[response] total_bytes_to_send: {:}", total_bytes_to_send);

//         if total_bytes_to_send == 0 {
//             eprintln!("[response] file is empty: {}", file_path);
//             self.status(404);
//             return Result::Err(());
//         }

//         let mime_type = mime_type(&file_path);
//         println!("[response] mime-type: {:?}", mime_type);

//         self.header("Transfer-Encoding", "chunked");
//         self.headers.set_content_length(total_bytes_to_send);
//         self.headers.set_content_type(mime_type);
//         self.status(200);

//         let mut response = self.headers.write();
//         response.push_str("\r\n");

//         // NOTE: Start writing the response to the client

//         let mut stream = &self.stream;
//         let _ = stream.write(response.as_bytes());
//         let _ = stream.write(&bytes);
//         let res = stream.write("\r\n".as_bytes());
//         println!("response: {:?}", res.is_ok());
//         let _ = stream.flush();
//         let _ = stream.shutdown(std::net::Shutdown::Both);

//         return Result::Ok(());
//     }

//     /**

//         Send the response to the client, consumes the stream.
//     */
//     pub fn send(&mut self) -> Result<(), Error> {
//         let body = self.body.as_str();
//         self.headers.set_content_length(body.len());
//         let mut response = self.headers.write();
//         response.push_str("\r\n");
//         response.push_str(body);
//         response.push_str("\r\n");

//         let bytes = response.as_bytes();
//         let mut stream = &self.stream;
//         println!("\r\n{:}", response);
//         let output = stream.write_all(bytes);
//         println!("[response] success: {:}", output.is_ok());
//         stream.flush();
//         stream.shutdown(std::net::Shutdown::Both);
//         output
//     }

//     fn response_empty(&mut self) -> String {
//         let content_length = self.headers.get("Content-Length".to_string());
//         if content_length.is_none() {
//             self.headers.set_content_length(0);
//         }
//         let mut empty_response = self.headers.write();
//         empty_response.push_str("\r\n");
//         empty_response
//     }

//     // fn response_with_body(&mut self) -> String {
//     //     println!(
//     //         "[response] response_with_body {}",
//     //         self.body.as_ref().unwrap().len()
//     //     );
//     //     let body = self.body.as_ref().unwrap();
//     //     self.headers.set_content_length(body.len());
//     //     let mut response = self.headers.write();
//     //     response.push_str("\r\n");
//     //     response.push_str(body);
//     //     response.push_str("\r\n");
//     //     response
//     // }
// }

// fn mime_type(file: &String) -> ContentType {
//     if file.ends_with(".html") {
//         return ContentType::HTML;
//     } else if file.ends_with(".css") {
//         return ContentType::CSS;
//     } else if file.ends_with(".png") {
//         return ContentType::PNG;
//     } else if file.ends_with(".json") {
//         return ContentType::JSON;
//     } else if file.ends_with(".txt") {
//         return ContentType::TEXT;
//     } else {
//         return ContentType::ANY;
//     }
// }
