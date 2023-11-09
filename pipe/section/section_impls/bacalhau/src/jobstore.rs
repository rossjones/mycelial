use crate::StdError;
use handlebars::Handlebars;
use serde_json::json;
use std::{collections::HashMap, fs, path::PathBuf};

pub struct JobStore {
    templates: Handlebars<'static>,
}

impl std::fmt::Debug for JobStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobStore").finish()
    }
}

impl JobStore {
    pub fn new(jobstore: impl Into<String>) -> Result<Self, StdError> {
        let path = PathBuf::from(jobstore.into());
        let entries = path.read_dir()?;

        let mut handlebars = Handlebars::new();
        for e in entries {
            let entry = e?;
            let meta = entry.metadata()?;
            if !meta.is_file() {
                continue;
            }

            let entryname = entry.file_name();
            let fname = entryname.to_str().unwrap();
            if fname.ends_with("yaml") || fname.ends_with("yml") {
                let p = entry.path();
                let name = p.file_stem().unwrap();

                handlebars.register_template_string(
                    name.to_str().unwrap(),
                    &*fs::read_to_string(entry.path())
                        .expect("should have been able to read the file"),
                )?;
            }
        }

        Ok(Self {
            templates: handlebars,
        })
    }

    pub fn render(&self, name: String, data: &HashMap<String, String>) -> Result<String, StdError> {
        self.templates
            .render(&name, &json!(data))
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Job;

    #[test]
    fn test_jobstore() -> Result<(), StdError> {
        let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "testdata"].iter().collect();

        let mut args: HashMap<String, String> = HashMap::new();
        args.insert("image".into(), "ubuntu".into());

        let j = JobStore::new(path.to_str().unwrap())?;
        let res = j.render(String::from("docker"), &args)?;

        // Parse yaml into a Job .
        let job: Job = serde_yaml::from_str(&res).unwrap();
        println!("Name: {:?}", job.name);

        let s = serde_json::to_string(&job).unwrap();
        println!("STR: {}", s);

        Ok(())
    }
}
