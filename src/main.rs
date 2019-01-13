extern crate regex;
extern crate slack;
extern crate yobot;

use std::env;
use regex::Regex;
use yobot::listener::{Message, MessageListener};
use yobot::Yobot;
use std::{thread, time};
use std::time::Duration;
use std::sync::mpsc::channel;

pub mod ast;
pub mod parser;
use parser::Parser;

pub struct EchoListener {
    regex: Regex,
}

impl EchoListener {
    pub fn new() -> EchoListener {
        EchoListener {
            regex: Regex::new(r".").unwrap(),
        }
    }
}

impl MessageListener for EchoListener {
    fn help(&self) -> String {
        String::from("`echo`: Just type anything")
    }

    fn re(&self) -> &Regex {
        &self.regex
    }

    fn only_when_addressed(&self) -> bool {
        true
    }

    fn handle(&self, message: &Message, cli: &slack::RtmClient) {
        let mut parser = Parser::new();
        let res = match parser.parse(&message.text) {
            Some(stat) => {
                let (sender, receiver) = channel();
                thread::spawn(move || {
                    sender.send(stat.get_expr().beta_reduction().to_string());
                });
                let mut mesg_opt = None;
                for _ in 0..100 {
                    thread::sleep(Duration::from_millis(10));
                    if let Ok(expr) = receiver.try_recv() {
                        mesg_opt = Some(expr);
                    }
                }
                if let Some(mesg) = mesg_opt {
                    mesg
                } else {
                    "time limit exceeded".to_string()
                }
            }
            None => "parse error".to_string(),
        };
        let _ = cli.sender().send_message(&message.channel, &res);
    }
}

fn main() {
    let token = match env::var("SLACK_BOT_TOKEN") {
        Ok(token) => token,
        Err(_) => panic!("Failed to get SLACK_BOT_TOKEN from env"),
    };
    let bot_name = match env::var("SLACK_BOT_NAME") {
        Ok(bot_name) => bot_name,
        Err(_) => panic!("Failed to get SLACK_BOT_NAME from env"),
    };

    let mut yobot = Yobot::new();
    let listener = EchoListener::new();
    yobot.add_listener(listener);
    yobot.connect(token, bot_name);
}
