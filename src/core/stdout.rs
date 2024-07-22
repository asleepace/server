use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, net::TcpStream, sync::Arc};

use super::file::Doc;

pub struct Stdout {
    connections: HashMap<String, Arc<TcpStream>>,
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

    pub fn add_connection(&mut self, stream: Arc<TcpStream>) {
        if let Ok(peer_addr) = stream.peer_addr() {
            println!("[stdout] new connection from: {}", peer_addr.to_string());
            self.connections.insert(peer_addr.to_string(), stream);
        }
    }

    pub fn write(&self, name: &str, data: String) {
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
