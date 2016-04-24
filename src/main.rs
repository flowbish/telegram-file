extern crate telegram_bot;
extern crate hyper;

use std::thread::spawn;
use std::env::var;
use std::io;
use std::fs::File;
use std::path::{Path,PathBuf};
use telegram_bot::{Api, MessageType, ListeningMethod, ListeningAction};
use telegram_bot::types::User;
use hyper::Url;
use hyper::method::Method;
use hyper::client::{Request};

const ENV_TOKEN: &'static str = "BOT_TOKEN";
const ENV_DOWNLOAD_DIR: &'static str = "DOWNLOAD_DIR";
const ENV_BASE_URL: &'static str = "BASE_URL";

fn download_file(download_dir: &Path, baseurl: &Url, url: &Url) -> io::Result<Url> {
    // Create a request to download the file
    let req = Request::new(Method::Get, url.clone()).unwrap();
    let mut resp = req.start().unwrap().send().unwrap();

    // Grab the last portion of the url
    let filename = url.path().unwrap().last().unwrap();

    // Create path by combining filename from url with download dir
    let mut path = download_dir.to_path_buf();
    path.push(filename);

    // Open file and copy downloaded data
    let mut file = try!(File::create(path));
    std::io::copy(&mut resp, &mut file).unwrap();

    // Create the return url that maps to this filename
    let mut returl = baseurl.clone();
    returl.path_mut().unwrap().push(filename.clone());
    Ok(returl)
}

fn download_file_user(user: &User, base_download_dir: &Path, base_url: &Url, url: &Url) -> io::Result<Url> {
    let mut download_dir_user = base_download_dir.to_path_buf();
    let user_path = user_path(&user);
    download_dir_user.push(user_path.clone());
    ensure_dir(&download_dir_user);
    let mut base_url_user = base_url.clone();
    base_url_user.path_mut().map(|p| p.push(user_path.clone()));
    download_file(&download_dir_user, &base_url_user, &url)
}

fn ensure_dir(path: &Path) {
    let _ = std::fs::create_dir(&path);
}

fn user_path(user: &User) -> String {
    match user.username.clone() {
        Some(name) => name.clone(),
        None => "anonymous".into()
    }
}

fn main() {
    let api = Api::from_env(ENV_TOKEN)
        .expect(&format!("Must set environment variable {}.", ENV_TOKEN));

    let download_dir = var(ENV_DOWNLOAD_DIR)
        .map(|s| PathBuf::from(s))
        .expect(&format!("Must set {} to a valid path", ENV_DOWNLOAD_DIR));

    let base_url = var(ENV_BASE_URL)
        .map(|s| Url::parse(&s))
        .expect(&format!("Must set {} to a valid url", ENV_BASE_URL)).unwrap();

    println!("getMe: {:?}", api.get_me());

    let mut listener = api.listener(ListeningMethod::LongPoll(None));

    ensure_dir(&download_dir);

    let tg_listener = spawn(move || {
        listener.listen(|u| {
            if let Some(m) = u.message {
                let user = m.from;

                let file_id = match m.msg.clone() {
                    MessageType::Photo(photos) => {
                        let largest_photo = photos.last().unwrap();
                        Some(largest_photo.file_id.clone())
                    },
                    MessageType::Sticker(sticker) => Some(sticker.file_id),
                    MessageType::Document(document) => Some(document.file_id),
                    MessageType::Audio(audio) => Some(audio.file_id),
                    MessageType::Video(video) => Some(video.file_id),
                    MessageType::Voice(voice) => Some(voice.file_id),
                    _ => None
                };

                // Handle media (files) sent to us directly.
                if let Some(file_id) = file_id {
                    let file = api.get_file(&file_id).unwrap();
                    if let Some(path) = file.file_path {
                        let tg_url = Url::parse(&api.get_file_url(&path)).unwrap();
                        let client_url = download_file_user(&user, &download_dir, &base_url, &tg_url).unwrap();
                        let _ = api.send_message(
                            m.chat.id(),
                            format!("{}", client_url),
                            None, None, None, None).unwrap();
                    }
                }

                // Handle URLs sent to us for rehosting.
                 if let MessageType::Text(txt) = m.msg {
                     if let Ok(url) = Url::parse(&txt) {
                         let client_url = download_file_user(&user, &download_dir, &base_url, &url).unwrap();
                         let _ = api.send_message(
                             m.chat.id(),
                             format!("{}", client_url),
                             None, None, None, None).unwrap();
                     }
                 }
            }
            Ok(ListeningAction::Continue)
        }).unwrap();
    });

    println!("Handling telegram requests!");

    tg_listener.join().unwrap();
}
