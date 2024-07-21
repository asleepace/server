use std::fs;

#[derive(Clone, Debug)]
pub struct URI {
    pub path: String,
    pub meta: Option<fs::Metadata>,
    pub exists: bool,
}

const FILTERS: [&str; 3] = ["..", "~", "."];

impl URI {
    pub fn clone(&self) -> URI {
        URI::new(self.path())
    }

    /**
        Create a new URI instance with the specified path.
    */
    pub fn new(path: &str) -> Self {
        let path = path.trim().to_string();
        let meta = match fs::metadata(&path) {
            Ok(meta) => Some(meta),
            Err(_) => None,
        };

        let exists = match &meta {
            Some(data) => data.is_file(),
            None => false,
        };

        URI { path, meta, exists }
    }

    /**
        Get the URI as a string.
    */
    pub fn to_string(&self) -> String {
        self.path.clone()
    }

    /**
        Format the specified path as a public path, this method will remove any
        filters from the path and return a new URI instance.
    */
    pub fn public(path: &str) -> URI {
        let trimmed = path.trim();
        let mut components = trimmed.split("/").collect::<Vec<&str>>();

        components = components
            .into_iter()
            .filter(|&x| !FILTERS.contains(&x))
            .collect();

        let public_path = format!("./src/public/{}", components.join("/"));
        println!("[uri] public path: {}", public_path.as_str());
        URI::new(&public_path)
    }

    /**
        Get the metadata for the file at the specified path, if the metadata
        is not available this method will return None.
    */
    pub fn meta_data(&self) -> Option<fs::Metadata> {
        match &self.meta {
            Some(data) => Some(data.clone()),
            None => {
                let meta = fs::metadata(&self.path);
                match meta {
                    Ok(data) => Some(data),
                    Err(_) => None,
                }
            }
        }
    }

    /**
       Get the path for the URI instance.
    */
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn is_file(&self) -> bool {
        match self.meta_data() {
            Some(data) => data.is_file(),
            None => false,
        }
    }

    /**
        Open the file at the specified path and return a file instance, if the
        file does not exist this method will return None.
    */
    pub fn open(&self) -> Option<Vec<u8>> {
        match fs::read(self.path()) {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    }

    /**
        Check if the file at the specified path exists.
    */
    pub fn exists(&self) -> bool {
        match self.meta_data() {
            Some(data) => data.is_file(),
            None => false,
        }
    }
}
