pub struct Config {
    pub host: String,
    pub port: u16,
}

//  IMPLEMENTATION: Write additional logic here for reading config
//  settings from a file or environment variables.
impl Config {
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn print(&self) {
        println!("[config] host: {}", self.host);
        println!("[config] port: {}", self.port);
    }
}

//  CONFIG: All server configuration settings are stored here.
//  This makes it easy to change settings without having to
//
pub fn server() -> Config {
    let host = "localhost";
    let port = 8080;
    Config {
        host: host.to_owned(),
        port: port,
    }
}
