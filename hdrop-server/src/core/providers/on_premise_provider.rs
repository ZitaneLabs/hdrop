use super::provider::Fetchtype;
use super::provider::StorageProvider;
use async_trait::async_trait;
//use anyhow::Result;
use crate::Result;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::{env, fs};

pub struct OnPremiseProvider<'a> {
    path: &'a str,
}

impl<'a> OnPremiseProvider<'a> {
    pub fn new(path: &'a str) -> Self {
        OnPremiseProvider { path }
    }
}

impl Default for OnPremiseProvider<'_> {
    fn default() -> Self {
        OnPremiseProvider {
            path: "/uploaded_files",
        }
    }
}

#[async_trait]
impl StorageProvider for OnPremiseProvider<'_> {
    async fn store_file(&self, ident: String, content: &[u8]) -> Result<String> {
        let file_path = format!("{path}/{ident}", path = self.path);

        let f = File::create(&file_path).expect("Unable to create file");
        let mut f = BufWriter::new(f);
        f.write_all(content).expect("Unable to write data");

        Ok(file_path)
    }

    async fn delete_file(&self, ident: String) -> Result<()> {
        let file_path = format!("{path}/{ident}", path = self.path);
        fs::remove_file(file_path).expect("Unable to delete file");
        Ok(())
    }

    async fn get_file(&self, ident: String) -> Result<Fetchtype> {
        let mut data = vec![];
        let file_path = format!("{path}/{ident}", path = self.path);
        let f = File::open(file_path).expect("Unable to open file");
        let mut br = BufReader::new(f);
        let r = br.read(&mut data).expect("Unable to read bytes"); // ToDo: this is not read_to_end(&mut data), change later to a proper impl
        println!("{r}");
        Ok(Fetchtype::FileData(data))
    }
}
