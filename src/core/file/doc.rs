use crate::core::file::URI;
use crate::core::util::get_mime_type;
use std::error::Error;
use std::fs;
use std::fs::FileType;

pub struct Doc {
    uri: URI,
    mime: String,
    data: Option<Vec<u8>>,
}

impl Doc {
    /**
        Read the contents of a file at the given URL. If the file does not exist
        this method will return an error.
    */
    pub fn read(url: &str) -> Result<Vec<u8>, ()> {
        return match fs::read(url) {
            Ok(data) => Ok(data),
            Err(err) => {
                eprintln!("[response] file not found: {} error: {}", url, err);
                return Result::Err(());
            }
        };
    }

    /**
        Open a file at the given URL and return a new Doc instance, the recommended
        way to open format a file is: "./index.html" with a "./" prefix, otherwise
        this file will attempt to format the file path as "./{url}".
    */
    pub fn open(path: &str) -> Option<Doc> {
        let uri = URI::new(path);
        let mime = match uri.meta_data() {
            Some(meta) => {
                let file_type = meta.file_type();
                let mime_type = format!("{:?}", file_type);
                println!("[doc] file type: {:?}", mime_type);
                mime_type
            }
            None => {
                eprintln!("[doc] error: unable to read file meta data");
                return None;
            }
        };

        let doc = Doc {
            uri,
            mime,
            data: None,
        };

        return Some(doc);
    }

    /**
        Read the contents of the file at the given URL and store the data in the
        Doc instance.
    */
    pub fn data(&mut self) -> Option<Vec<u8>> {
        if self.data.is_some() {
            match self.data.clone() {
                Some(data) => return Some(data),
                None => {
                    eprintln!("[doc] error: failed to read file data");
                    // fall through
                }
            }
        }

        self.uri.open()
    }
}
