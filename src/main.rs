extern crate telegram_bot;

use std::thread::spawn;
use std::env::var;
use std::io::Result;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver, channel};
use telegram_bot::{Api, MessageType, ListeningMethod, ListeningAction};

const ENV_TOKEN: &'static str = "TGIMG_BOT_TOKEN";
const ENV_MEDIA_DIR: &'static str = "TGIMG_MEDIA_DIR";
const ENV_HOST_PREFIX: &'static str = "TGIMG_HOST_PREFIX";

fn main() {
    let api = match Api::from_env(ENV_TOKEN) {
        Ok(api) => api,
        Err(_) => {
            println!("Must set environment variable {}.", ENV_TOKEN);
            std::process::exit(1);
        }
    };

    println!("getMe: {:?}", api.get_me());

    let mut listener = api.listener(ListeningMethod::LongPoll(None));

    let tg_listener = spawn(move || {
        listener.listen(|u| {
            if let Some(m) = u.message {
                let user = m.from;
                match m.msg {
                    MessageType::Text(txt) => { println!("{} sent message \"{}\"", user.first_name, txt); }
                    MessageType::Photo(photos) => {
                        let largest_photo = photos.last().unwrap();
                        let message = api.get_file(&largest_photo.file_id).unwrap();
                        println!("{:?}", message);
                    }
                    _ => {}
                }
                // if let MessageType::Text(txt) = m.msg {
                //     println!("{:?}", txt);
                // }
            }
            Ok(ListeningAction::Continue)
        }).unwrap();
    });

    println!("Handling telegram requests!");

    tg_listener.join().unwrap();
}
