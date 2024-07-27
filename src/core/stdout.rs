use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, net::TcpStream, sync::Arc};

use super::data::ServerEvent;
use super::file::Doc;
use super::http::HttpRequest;

pub struct Stdout {
    connections: HashMap<String, HttpRequest>,
    output_file: String,
    environment: String,
}

impl Stdout {
    pub fn new(output_file: &str, environment: &str) -> Self {
        Stdout {
            connections: HashMap::new(),
            output_file: output_file.to_string(),
            environment: environment.to_string(),
        }
    }

    pub fn add_stream(&mut self, stream: HttpRequest) {
        let stream_name = stream.uri.clone();
        println!("[stdout] adding stream: {}", stream_name);
        self.connections.insert(stream_name, stream);
    }

    pub fn write(&mut self, name: &str, data: String) {
        // Write data to csv file.
        // log_time, log_type, log_name, log_host, log_data
        // 2021-09-01 12:00:00, environment, name, data
        let mut file = match OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.output_file)
        {
            Ok(csv_file) => csv_file,
            Err(err) => {
                eprintln!(
                    "[stdout] failed to open file: {} {:}",
                    self.output_file, err
                );
                return;
            }
        };

        let csv_line = format!(
            "{},{},{},{}\n",
            Self::timestamp(),
            self.environment,
            name,
            data
        );

        let raw_bytes = csv_line.as_bytes();
        match file.write_all(raw_bytes) {
            Ok(_) => println!("[stdout] wrote to file: {}", self.output_file),
            Err(err) => eprintln!("[stdout] failed to write to file: {}", err),
        }

        // Create server event from which will be broadcasted to all streams.
        let event = ServerEvent::event("base64", data.to_string());

        // println!(
        //     "[stdout] total connections: {} total",
        //     self.connections.len()
        // );

        // Iterate connections sending server event to each stream, and closing streams that fail.
        self.connections
            .retain(|_, stream| match stream.server_side_event(event.clone()) {
                Ok(_) => {
                    println!("[stdout] sent event to stream: {}", stream.uri);
                    true
                }
                Err(err) => {
                    eprintln!("[stdout] failed to send event: {}", err);
                    false
                }
            });
    }

    fn keep_alive(&mut self) {
        let event = ServerEvent::event("keep-alive", "ping".to_string());
        self.connections
            .retain(|_, stream| match stream.server_side_event(event.clone()) {
                Ok(_) => true,
                Err(err) => {
                    eprintln!("[stdout] failed to keep stream alive: {}", err);
                    false
                }
            });
    }

    fn timestamp() -> String {
        let now = SystemTime::now();
        let duration = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();

        let (year, month, day, hour, min, sec) = {
            let secs = secs + 62_167_219_200; // Seconds from year 0 to 1970
            let years = secs / 31_557_600; // Approximate seconds per year
            let year = 1970 + years;
            let remainder = secs % 31_557_600;
            let month = remainder / 2_629_800; // Approximate seconds per month
            let day = (remainder % 2_629_800) / 86400;
            let hour = (remainder % 86400) / 3600;
            let min = (remainder % 3600) / 60;
            let sec = remainder % 60;
            (year, month + 1, day + 1, hour, min, sec)
        };

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:09}Z",
            year, month, day, hour, min, sec, nanos
        )
    }
}
