use std::collections::LinkedList;

use chrono::Local;
use serde::{Serializer, Serialize, ser::SerializeStruct};


enum Color {
    Red,
}

struct Message {
    id: String,
    color: Color,
    content: String,
}

struct Buffer {
    queue: LinkedList<Message>,
    config: BufferConfig,
}

struct BufferConfig {
    max_size: u32,
}

pub fn danmu(name: &str, content: &str) {
    println!("{} -- {} : {}", Local::now(), name, content);
}

pub fn gift(name: &str, gift: &str, num: u64) {
    println!(
        "{} -- 感谢 {} 送出的 {} 个 {}！",
        Local::now(),
        name,
        num,
        gift
    );

}

pub fn welcome(name: &str) {
    println!("{} -- 欢迎 {} 来到直播间！", Local::now(), name);
    send_message(format!("欢迎 {} 来到直播间！", name));
}

pub fn guard(name: &str){

}
struct message{
    route:String,
    frameqq:String,
    friendsqq:String,
    newscontent:String,
}
//为message实现序列化
impl Serialize for message{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer{
            let mut state = serializer.serialize_struct("message",4)?;
            state.serialize_field("route",&self.route)?;
            state.serialize_field("frameqq",&self.frameqq)?;
            state.serialize_field("friendsqq",&self.friendsqq)?;
            state.serialize_field("newscontent",&self.newscontent)?;
            state.end()
        }
}

pub async fn send_message(message:String){
    let url = "http://172.22.47.167:8090/send/friends";
    let client = reqwest::Client::new();
    let send = message{
        route:"53".to_string(),
        frameqq:"2738373735".to_string(),
        friendsqq:"2594574739".to_string(),
        newscontent:message,
    };
    let res = client.post(url)
        .json(&send)
        .send()
        .await;

}