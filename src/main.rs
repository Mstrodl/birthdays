use chrono::offset::Local;
use chrono::Datelike;
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use std::vec::Vec;

use birthdays::ldap::client as ldap_client;

#[tokio::main]
async fn main() {
    let LDAP_DN = env::var("LDAP_BIND_DN").unwrap();
    let LDAP_PW = env::var("LDAP_BIND_PW").unwrap();
    let SLACK_URL = env::var("SLACK_URL").unwrap_or("http://localhost:8080".to_string());
    let mut ldap_client = ldap_client::LdapClient::new(
        &LDAP_DN,
        &LDAP_PW,
    ).await;
    let d = Local::today().naive_local();
    let date_string = format!("{:02}{:02}", &d.month(), &d.day());
    let members = ldap_client.search_birthday(&date_string).await;
    let mut users: Vec<String> = Vec::new();
    for member in members {
        if &member.birthday[4..8] == &date_string {
            users.push(match member.slackuid {
                Some(uid) => format!("<@{}>", uid),
                None => member.cn,
            });
        }
    }
    let mut output = String::from("Happy Birthday to ");
    match users.len() {
        0 => output.push_str("no one. :sad-bidoof:");
        1 => output.push_str(&format!("{}!", users[0]));
        2 => output.push_str(&format!("{} and {}!", users[0], users[1]));
        user_count => {
            for i in [..user_count-1] {
                let s = &users[i];
                output.push_str(&format!("{}, ", s));
            }
            output.push_str(&format!("and {}!", users[user_count-1]));
        }
    }
    let mut map = HashMap::new();
    //map.insert("channel", "CBBK03MQ9");
    map.insert("text", &output);
    let client = Client::new();
    println!("{}", output);
    let res = client.post(SLACK_URL)
        .header("Content-type", "application/json")
        .json(&map)
        .send()
        .await;
    //let res = client.post("https://slack.com/api/chat.postMessage")
    //                .header("Content-type", "application/json")
    //                .header("Authorization", &format!("Bearer {}", SLACK_TOKEN))
    //                .json(&map)
    //                .send()
    //                .await;
}

