// pub struct Config {
//     pub host: String,
//     pub port: u16,
// }

// /**
//     Configuration for the server.

// */
// impl Config {
//     pub fn new(host: &str, port: u16) -> Self {
//         Config {
//             host: host.to_string(),
//             port,
//         }
//     }

//     pub fn address(&self) -> String {
//         format!("{}:{}", self.host, self.port)
//     }

//     pub fn print(&self) {
//         println!("[config] host: {}", self.host);
//         println!("[config] port: {}", self.port);
//     }

//     pub fn public(&self, path: &str) -> String {
//         let asset_path = format!("./public/{}", path);
//         println!("[confi] public: {}", asset_path);
//         asset_path
//     }
// }
