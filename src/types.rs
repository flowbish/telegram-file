use error::{Result,Error};

use std::io;
use std::fs;
use std::path::{Path,PathBuf};
use hyper::client::{Request,Response};
use hyper::method::Method;
use telegram_bot;
use telegram_bot::Api;

pub use hyper::Url;

trait TelegramFileDownloader {
    fn download_file(&self, file_id: &str) -> File;
}

impl TelegramFileDownloader for Api {
    fn download_file(&self, file_id: &str) -> File {
        File::new(self, file_id)
    }
}

/// Represents a file from telegram
pub struct File {
    url: Url,
}

fn telegram_url(api: &Api, file_id: &str) -> Result<Url> {
    let file = api.get_file(file_id).unwrap();
    if let Some(path) = file.file_path {
        let url = api.get_file_url(&path);
        Ok(Url::parse(&url).unwrap())
    }
    else {
        Err(Error::from("test"))
    }
}

impl File {
    pub fn new(api: &Api, file_id: &str) -> File {
        let url = telegram_url(api, file_id).unwrap();
        File{ url: url }
    }

    pub fn filename(&self) -> Result<String> {
        filename(&self.url)
    }

    pub fn download_to_path(&mut self, path: &Path) -> Result<PathBuf> {
        // Open file and copy downloaded data
        let mut file = try!(fs::File::create(path));
        let mut resp = download(&self.url);
        try!(io::copy(&mut resp, &mut file));
        Ok(path.to_path_buf())
    }

    pub fn download_to_dir(&mut self, path: &Path) -> Result<PathBuf> {
        ensure_dir(path);

        // Create path by combining filename from url with download dir
        let filename = try!(self.filename());
        let mut path = path.to_path_buf();
        path.push(&filename);

        self.download_to_path(&path)
    }
}

/// Represent a particular Telegram user
pub struct User {
    user: telegram_bot::types::User
}

impl User {
    pub fn display_name(&self) -> String {
        let mut name = self.user.first_name.clone();
        if let Some(ref last_name) = self.user.last_name {
            name.push(' ');
            name.push_str(&last_name);
        }
        name
    }

    pub fn username(&self) -> Option<String> {
        self.user.username.clone()
    }
}

fn ensure_dir(path: &Path) {
    let _ = ::std::fs::create_dir(&path);
}

fn filename(url: &Url) -> Result<String> {
    let path = try!(url.path().ok_or(Error::from("Url has no path")));
    let filename: &str = try!(path.last().ok_or(Error::from("Url has no path")));
    Ok(filename.into())
}

fn download(url: &Url) -> Response {
    // Create a request to download the file
    let req = Request::new(Method::Get, url.clone()).unwrap();
    let resp = req.start().unwrap().send().unwrap();

    resp
}
