#[macro_use] extern crate serenity;
extern crate dotenv;
extern crate websocket;
extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate futures;
extern crate percent_encoding;
extern crate typemap;

mod commands;
mod lavalink;
mod handler;

use lavalink::rest::HttpClient;
use lavalink::opcodes::Opcode;
use lavalink::socket::Socket;

use std::env;
use std::thread;
use std::time::Duration;

use serenity::client::CACHE;
use serenity::framework::StandardFramework;
use serenity::model::*;
use serenity::prelude::*;
use serenity::voice;
use serenity::Result as SerenityResult;
use dotenv::dotenv;
use websocket::OwnedMessage;
use tokio_core::reactor::Core;

fn main() {
    // load .env into environment variables
    let _ = dotenv();

    // getting environment variables
    let discord_token = env::var("DISCORD_TOKEN").unwrap();

    let lavalink_http_host = env::var("LAVALINK_HTTP_HOST").unwrap();
    let lavalink_websocket_host = env::var("LAVALINK_WEBSOCKET_HOST").unwrap();
    let lavalink_user_id = env::var("LAVALINK_USER_ID").unwrap();
    let lavalink_password = env::var("LAVALINK_PASSWORD").unwrap();
    let lavalink_num_shards = env::var("LAVALINK_NUM_SHARDS")
        .map(|num_shards| num_shards.parse::<i32>().unwrap()).unwrap();

    // start the lavalink socket!!
    let lavalink_socket = Socket::open(&lavalink_websocket_host, &lavalink_user_id, &lavalink_password,
                              lavalink_num_shards);

    // say join the voice channel lol
    let _ = lavalink_socket.ws_tx.send(OwnedMessage::Text(json!({
        "op": Opcode::Connect.to_string(),
        "guildId": "272410239947767808",
        "channelId": "320643590986399749",
    }).to_string()));

    // serenity!!!
    let mut client = Client::new(&discord_token, handler::Handler);

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .prefix("!")
            .on_mention(true))
        .on("stop", commands::admin::stop)
        .on("ping", commands::meta::ping)
        .on("join", commands::voice::join)
        .on("leave", commands::voice::leave));

    {
        let data = &mut *client.data.lock();
        let _ = data.insert::<commands::admin::CloseHandleKey>(client.close_handle().clone());
    }

    let _ = client.start()
        .map_err(|err| println!("serenity client ended: {:?}", err));

    // close the lavalink socket
    lavalink_socket.close();
}