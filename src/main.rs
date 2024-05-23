use chrono::offset::Local;
use chrono::Datelike;
use dotenvy::dotenv;
use reqwest::Client;
use serde::Serialize;
use std::env;
use std::vec::Vec;

use birthdays::ldap::client as ldap_client;

#[derive(Serialize, Debug, Clone)]
struct SendMessageRequest<'a> {
    text: &'a str,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let ldap_dn = env::var("LDAP_BIND_DN").unwrap();
    let ldap_pw = env::var("LDAP_BIND_PW").unwrap();
    let slack_url = env::var("SLACK_URL").unwrap_or("http://localhost:8080".to_string());
    let mut ldap_client = ldap_client::LdapClient::new(&ldap_dn, &ldap_pw).await;
    let today = Local::now().naive_local();
    let date_string = format!("{:02}{:02}", today.month(), today.day());
    let members = ldap_client.search_birthday(&date_string).await;
    let mut users: Vec<String> = Vec::new();
    for member in members {
        if member.birthday[4..8] == date_string {
            users.push(match member.slackuid {
                Some(uid) => format!("<@{}>", uid),
                None => member.cn,
            });
        }
    }
    let birthday_list = match users.len() {
        0 => "no one. :sad-bidoof:".to_owned(),
        1 | 2 => format!("{}!", users.join(" and ")),
        length => {
            format!(
                "{}, and {}!",
                users[0..length - 1].join(", "),
                users.last().unwrap()
            )
        }
    };
    let output = format!("Happy Birthday to {birthday_list}");
    println!("{}", output);

    let client = Client::new();
    client
        .post(slack_url)
        .header("Content-type", "application/json")
        .json(&SendMessageRequest { text: &output })
        .send()
        .await
        .unwrap();
}
