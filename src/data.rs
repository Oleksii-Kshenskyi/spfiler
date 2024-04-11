use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct ListFilesResponse {
    pub message: String,
    pub files: Option<Vec<String>>,
}

impl std::fmt::Display for ListFilesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.files {
            None => write!(f, "WHOOPS: Server doesn't know this ID!"),
            Some(vec) => {
                writeln!(f, "FILES:")?;
                if vec.is_empty() {
                    writeln!(f, "- <EMPTY>")?;
                    return Ok(());
                }
                for name in vec {
                    writeln!(f, "- `{}`;", name)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisteredResponse {
    pub id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
// The query for exiting the app
pub struct ExitResponse {
    pub response: String,
}
