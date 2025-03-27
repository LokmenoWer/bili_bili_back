use anyhow::Ok;
use bili_bili_back::{
    core::bilibili::{check_login, get_login, read_cookie},
    messages::danmu_msg::{DanmuMsg, GiftRoot, GUARD_BUY, INTERACT_WORD},
    Client, Message,
};

use chrono::format::format;
use qrcode::render::unicode;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write, time::Duration, vec, vec::Vec, path::Path};
use tokio::{
    fs::{OpenOptions, remove_file, File},
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::{self, sleep},
};
use url::Url;

#[tokio::main]
async fn main() {
    //尝试读取cookie文件

    let result = read_cookie();
    //根据result判断是否需要登录
    if result.1 == false {
        let login = get_login().await;
        let code = QrCode::new(login.url.clone()).unwrap();
        let string = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Dark)
            .light_color(unicode::Dense1x2::Light)
            .build();
        println!("{}", string);
        let mut flag = false;
        let mut result = check_login(&login).await;
        while flag == false {
            result = check_login(&login).await;
            flag = result.0;
            sleep(time::Duration::from_secs(5)).await;
        }
        let url = result.1;
        let url = Url::parse(&url).unwrap();
        let mut query_pairs = url.query_pairs();
        //初始化结构体
        let mut bilicookie = bili_bili_back::core::bilibili::bilib_cookie {
            DedeUserID: "".to_string(),
            DedeUserID__ckMd5: "".to_string(),
            Expires: "".to_string(),
            SESSDATA: "".to_string(),
            bili_jct: "".to_string(),
        };
        for (key, value) in query_pairs {
            println!("{}={}", key, value);
            //将cookie写入结构体
            match key.as_ref() {
                "DedeUserID" => bilicookie.DedeUserID = value.to_string(),
                "DedeUserID__ckMd5" => bilicookie.DedeUserID__ckMd5 = value.to_string(),
                "Expires" => bilicookie.Expires = value.to_string(),
                "SESSDATA" => bilicookie.SESSDATA = value.to_string(),
                "bili_jct" => bilicookie.bili_jct = value.to_string(),
                _ => (),
            }
        }

        //将bilicookie写入文件
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open("cookie.txt")
            .await
            .unwrap();
        let json = serde_json::to_string(&bilicookie).unwrap();
        file.write_all(json.as_bytes()).await.unwrap();
        println!("cookie写入成功");
    }
    tokio::join!(qq_linstener(), bili_linstener(), live_linstener());
}

async fn qq_linstener() {
    //接受qq消息上报
    let mut listener = TcpListener::bind("127.0.0.1:5701").await.unwrap();
    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buffer = [0u8; 1024];
        let n = socket.read(&mut buffer).await.unwrap();
        let raw = String::from_utf8_lossy(&buffer[..n]).to_string();
        let mut raw = raw.split("\n");
        for _ in 0..8 {
            raw.next();
        }
        let raw = raw.next().unwrap().to_owned();
        println!("{}", raw);
        let res = resolv_qq_msg(raw).await;

        //返回 HTTP 响应
        if res.to_string() == "" {
            socket
                .write_all("HTTP/1.1 204 No Content\r\n\r\n".as_bytes())
                .await
                .unwrap();
        } else {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                res.len(),
                res
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        }

        /*
        //如果res为success，返回状态码204
        if res == "success" {
            socket.write_all("HTTP/1.1 204 No Content\r\n\r\n".as_bytes())
                .await
                .unwrap();

        } else {
            //Content-Type设置为application/json
            socket.write_all(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n"
                    .as_bytes(),
            );
            //发送时发送json原始数据
            socket.write_all(res.as_bytes()).await.unwrap();
        } */

        println!("{}", res);

        //打印消息
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub time: Option<i64>,
    pub self_id: Option<i64>,
    pub post_type: Option<String>,
}
pub type qq_message_root = Vec<qq_message_root2>;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct qq_message_root2 {
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub data: Option<qq_Data>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct qq_Data {
    pub qq: Option<i64>,
    pub text: Option<String>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sender {
    pub user_id: i64,
    pub nickname: String,
    pub sex: String,
    pub age: i32,
    pub group_id: Option<i64>,
    pub card: Option<String>,
    pub area: Option<String>,
    pub level: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event_Message {
    pub Event: Option<Event>,
    pub message_type: Option<String>,
    pub sub_type: Option<String>,
    pub message_id: Option<i64>,
    pub user_id: Option<i64>,
    pub message: Option<String>,
    pub raw_message: Option<String>,
    pub font: Option<i32>,
    pub sender: Option<Sender>,
    pub target_id: Option<i64>,
    pub temp_source: Option<i64>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event_Request {
    pub Event: Event,
    pub request_type: String,
    pub user_id: Option<i64>,
    pub comment: Option<String>,
    pub flag: Option<String>,
    pub sub_type: Option<String>,
    pub group_id: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event_Notice {
    pub notice_type: Option<String>,
    pub time: Option<i64>,
    pub self_id: Option<i64>,
    pub post_type: Option<String>,
    pub sub_type: Option<String>,
    pub group_id: Option<i64>,
    pub operator_id: Option<i64>,
    pub user_id: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct quick_reply {
    pub reply: String,
    pub auto_escape: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct quick_control {
    pub approve: bool,
    pub reason: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupRequest {
    pub post_type: Option<String>,
    pub request_type: Option<String>,
    pub time: Option<i64>,
    pub self_id: Option<i64>,
    pub sub_type: Option<String>,
    pub group_id: Option<i64>,
    pub user_id: Option<i64>,
    pub invitor_id: Option<i64>,
    pub comment: Option<String>,
    pub flag: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupNotice {
    pub post_type: Option<String>,
    pub notice_type: Option<String>,
    pub time: Option<i64>,
    pub self_id: Option<i64>,
    pub sub_type: Option<String>,
    pub user_id: Option<i64>,
    pub group_id: Option<i64>,
    pub operator_id: Option<i64>,
}

async fn resolv_qq_msg(raw: String) -> String {
    //使用库进行URL解码
    let raw = urlencoding::decode(&raw).unwrap();
    //将\"替换为"
    let raw = raw.replace("\\\"", "\"");
    //将\"替换为"
    let raw = raw.replace("\\\\", "\\");
    //按照Event粗步解析Json为Event结构体
    let event = match serde_json::from_str::<Event>(&raw) {
        std::result::Result::Ok(t) => t,
        Err(e) => {
            println!("解析Json出错，原始数据为：{}", raw);
            println!("错误信息为：{}", e);
            return "".to_string();
        }
    }; 
    match event.post_type.unwrap().as_str() {
        "message" => {
            let event_message = serde_json::from_str::<Event_Message>(&raw).unwrap();
            if event_message.message_type.unwrap() == "private" {
                //如果消息内容包含"直播间状态"，则返回直播间状态
                if event_message.message.unwrap().contains("直播间状态") {
                    //构建快速回复
                    let mut res = quick_reply::default();
                    if get_live_status(23353816).await.0 {
                        res.reply =
                            "直播间正在直播\n链接：https://live.bilibili.com/23353816".to_string();
                    } else {
                        res.reply = "直播间未开播,不过您可以先关注下主播\n链接：https://live.bilibili.com/23353816".to_string();
                    }
                    let res = serde_json::to_string(&res).unwrap();
                    //去除空格
                    let res = res.replace(" ", "");
                    return res;
                } else {
                    //私聊消息
                    //构建快速回复

                    return "".to_owned();
                }
            } else {
                //群消息
                //构建快速回复
                if event_message.message.unwrap().contains("直播间状态") {
                    //构建快速回复
                    let mut res = quick_reply::default();
                    if get_live_status(23353816).await.0 {
                        res.reply =
                            "直播间正在直播\n链接：https://live.bilibili.com/23353816".to_string();
                    } else {
                        res.reply = "直播间未开播,不过您可以先关注下主播\n链接：https://live.bilibili.com/23353816".to_string();
                    }
                    let res = serde_json::to_string(&res).unwrap();
                    //去除空格
                    let res = res.replace(" ", "");
                    return res;
                } else {
                    return "".to_owned();
                }
            }
        }

        "request" => {
            let event_request = serde_json::from_str::<GroupRequest>(&raw).unwrap();
            if event_request.request_type.unwrap() == "group" {
                //群请求
                //构建快速回复
                let event_request = serde_json::from_str::<GroupRequest>(&raw).unwrap();

                try_set_group_add_request(
                    event_request.flag.unwrap(),
                    event_request.sub_type.unwrap(),
                )
                .await;
                return "".to_owned();
            } else {
                return "".to_owned();
            }
        }
        "notice" => {
            let event_notice = serde_json::from_str::<GroupNotice>(&raw).unwrap();
            if event_notice.notice_type.unwrap() == "group_increase" {
                let msg = format!(
                    "欢迎新人[CQ:at,qq={}]加入本群,请多多一键三连哦",
                    event_notice.user_id.unwrap()
                );
                //启动一个新的线程发送消息
                tokio::spawn(async move {
                    try_send_group_only(msg, event_notice.group_id.unwrap()).await;
                });
                return "".to_owned();
                

                
            } else {
                //忽略
                return "".to_string();
            }
        }
        _ => {
            //忽略
            return "".to_string();
        }
    }
    let event = serde_json::from_str::<GroupNotice>(&raw).unwrap();
    match event.notice_type.unwrap().as_str() {
        "group_increase" => {
            let msg = format!(
                "欢迎新人[CQ:at,qq={}]加入本群,请多多一键三连哦",
                event.user_id.unwrap()
            );
            try_send_group_only(msg, event.group_id.unwrap()).await;
            return "".to_owned();
        }
        _ => {
            //忽略
            return "".to_string();
        }
    }
}

async fn try_set_group_add_request(flag: String, sub_type: String) {
    //尝试发送消息
    let mut i = 0;
    loop {
        if set_group_add_request(flag.clone(), sub_type.clone()).await {
            break;
        }
        i += 1;
        if i > 5 {
            break;
        }
    }
}

async fn set_group_add_request(flag: String, sub_type: String) -> bool {
    let url = "http://127.0.0.1:5700/set_group_add_request";
    let client = reqwest::Client::new();
    let res = match client
        .post(url)
        .json(&serde_json::json!({
            "flag": flag,
            "sub_type": sub_type,
            "approve":true,
            "reason":"",
        }))
        .send()
        .await
    {
        std::result::Result::Ok(res) => res,
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };
    true
}

async fn bili_linstener() {
    let room_id = 23353816;
    let mut client = Client::new_anonymous(room_id).await.unwrap();
    println!("Room {} Connected", room_id);
    loop {
        let message = client.next().await.unwrap();
        match message {
            Message::OpHeartbeatReply(v) => {
                println!("Room {} Popularity: {}", room_id, v.popularity);
            }
            Message::OpMessage(v) => match v.cmd.as_str() {
                "DANMU_MSG" => {
                    let v = serde_json::from_slice::<DanmuMsg>(&v.data).unwrap();
                    println!(
                        "Room {} 弹幕: [{}|{}] {}: {}",
                        room_id,
                        v.fans_medal_name(),
                        v.fans_medal_level(),
                        v.uname(),
                        v.msg()
                    );
                    try_send(format!("{}:\n{}", v.uname(), v.msg())).await;
                }
                "SEND_GIFT" => {
                    let v = serde_json::from_slice::<GiftRoot>(&v.data).unwrap();
                    println!("{}送出了{}个{}", v.data.uname, v.data.num, v.data.gift_name);
                    try_send(format!(
                        "{}送出了{}个{}",
                        v.data.uname, v.data.num, v.data.gift_name
                    ))
                    .await;
                }
                "GUARD_BUY" => {
                    let v = serde_json::from_slice::<GUARD_BUY>(&v.data).unwrap();
                    println!("{}购买了{}", v.data.uname, v.data.gift_name);
                    try_send(format!("{}购买了{}", v.data.uname, v.data.gift_name)).await;
                }
                "INTERACT_WORD" => {
                    let v = serde_json::from_slice::<INTERACT_WORD>(&v.data).unwrap();
                    println!("{}\t进入直播间", v.data.uname);
                    if if_welcome(v.data.uid) {
                        try_send(format!("{}\t进入直播间", v.data.uname)).await;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn if_welcome(uid: i64) -> bool {
    let attention_list: Vec<u64> = vec![
        1945547176, 42915582, 21240539, 485501696, 8668075, 272349920, 1771491189, 5806635,
        27496981, 367407934, 249467591, 473242906, 397146821, 8235987, 361706310, 174560331,
        475917494, 34705748, 177355925, 24326458, 628884258, 20980811, 400122979, 673992519,
        288203400, 3786388, 40123369, 557711464, 31219375, 34997646, 18124018, 274662200, 6431974,
        217675150, 321502334, 702439037, 2048746870, 37786427, 11354515, 34070485, 101078521,
        275295805, 29195754, 440727948, 13335795, 1862855, 474551893, 518761546, 402821078,
        2376474, 1984454684, 691261277, 576592456, 343503720, 516266029, 266958223, 323304528,
    ];
    for i in attention_list {
        if i == uid as u64 {
            return true;
        }
    }
    return false;
}

async fn try_send(msg: String) {
    //尝试发送消息
    let mut i = 0;
    loop {
        if send_to_qq(msg.clone()).await {
            break;
        }
        i += 1;
        if i > 5 {
            break;
        }
    }
}

async fn send_to_qq(msg: String) -> bool {
    let url = "http://127.0.0.1:5700/send_private_msg";
    let send_list: Vec<i64> = vec![1694865639, 2594574739];
    for i in send_list {
        let client = reqwest::Client::new();
        let res = match client
            .post(url)
            .json(&serde_json::json!({
                "user_id": i,
                "message": msg,
            }))
            .send()
            .await
        {
            std::result::Result::Ok(res) => res,
            Err(e) => {
                println!("Error: {}", e);
                return false;
            }
        };
        println!("{:?}", res.text().await.unwrap());
    }
    true
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveStatus {
    pub code: i64,
    pub msg: String,
    pub message: String,
    pub data: live_Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct live_Data {
    pub uid: i64,
    pub room_id: i64,
    pub short_id: i64,
    pub attention: i64,
    pub online: i64,
    pub is_portrait: bool,
    pub description: String,
    pub live_status: i64,
    pub area_id: i64,
    pub parent_area_id: i64,
    pub parent_area_name: String,
    pub old_area_id: i64,
    pub background: String,
    pub title: String,
    pub user_cover: String,
    pub keyframe: String,
    pub is_strict_room: bool,
    pub live_time: String,
}

async fn live_linstener() {
    //q:如何使用lock文件来判断是否已经发送过直播开始消息
    //a:如果存在lock文件，说明已经发送过直播开始消息，直接进入直播间监听
    //a:如果不存在lock文件，说明没有发送过直播开始消息，先发送直播开始消息，再进入直播间监听
    loop {
        let live_status = get_live_status(23353816).await;
        if live_status.0 {
            // Check if live_status.1 is different from the contents of the lock file
            let broadcast_message = live_status.1.clone();
            let mut file = File::open("lock").await;
            let mut lock_contents = String::new();
            if let std::result::Result::Ok(mut f) = file {
                f.read_to_string(&mut lock_contents).await.unwrap();
            }
            if lock_contents != live_status.1 {
                // Send broadcast and write live_status.1 to the lock file
                send_broadcast(live_status.1).await;
                let mut file = File::create("lock").await.unwrap();
                file.write_all(broadcast_message.as_bytes()).await.unwrap();
            }
            loop {
                let live_status = get_live_status(23353816).await;
                if !live_status.0 {
                    // If live status is false, delete the lock file and break the loop
                    if Path::new("lock").exists() {
                        remove_file("lock").await.unwrap();
                    }
                    try_send_group("[直面泰山] 下播啦~".to_owned()).await;
                    break;
                }
                sleep(Duration::from_secs(60)).await;
            }
        } else {
            loop {
                let live_status = get_live_status(23353816).await;
                if live_status.0 {
                    break;
                }
                sleep(Duration::from_secs(60)).await;
            }
        }
    }
}

async fn get_live_status(roomid: i64) -> (bool, String) {
    let url = format!(
        "https://api.live.bilibili.com/room/v1/Room/get_info?room_id={}",
        roomid
    );
    let client = reqwest::Client::new();
    let res = match client.get(url).send().await {
        std::result::Result::Ok(res) => res,
        Err(e) => {
            println!("Error: {}", e);
            return (false, "".to_owned());
        }
    };
    let res = res.text().await.unwrap();
    let res = urlencoding::decode(&res).unwrap();
    //将\"替换为"
    let res = res.replace("\\\"", "\"");
    //将\"替换为"
    let res = res.replace("\\\\", "\\");

    let res = serde_json::from_str::<LiveStatus>(&res).unwrap();
    println!("{:?}", res.data.live_status);
    if res.data.live_status == 0 {
        return (false, "".to_owned());
    }
    return (true, res.data.live_time);
}

async fn send_broadcast(time: String) {
    /*[直面泰山]的直播间开播啦！！！
    [图片]
    公告：直播时间 上午10-1点  下午7-10点   ios充值公众号搜索哔哩哔哩直播
    开播时间：18:07:10
    链接：https://live.bilibili.com/23353816 */
    let msg = format!("[直面泰山]的直播间开播啦！！！\n[CQ:image,file=https://img1.imgtp.com/2023/06/08/NsG1CvFo.jpg]\n公告：直播时间 上午10-1点  下午7-10点   ios充值公众号搜索哔哩哔哩直播\n开播时间：{}\n链接：https://live.bilibili.com/23353816",time);
    try_send_group(msg).await;
}

async fn try_send_group(msg: String) {
    //尝试发送消息
    let mut i = 0;
    loop {
        if send_to_group(msg.clone()).await {
            break;
        }
        i += 1;
        if i > 5 {
            break;
        }
    }
}

async fn try_send_group_only(msg: String, group_id: i64) {
    //尝试发送消息
    let mut i = 0;
    loop {
        if send_to_group_only(msg.clone(), group_id).await {
            break;
        }
        i += 1;
        if i > 5 {
            break;
        }
    }
}
async fn send_to_group_only(msg: String, group_id: i64) -> bool {
    let url = "http://127.0.0.1:5700/send_group_msg";

    let client = reqwest::Client::new();
    let res = match client
        .post(url)
        .json(&serde_json::json!({
            "group_id": group_id,
            "message": msg,
        }))
        .send()
        .await
    {
        std::result::Result::Ok(res) => res,
        Err(e) => {
            println!("Error: {}", e);
            return false;
        }
    };
    println!("{:?}", res.text().await.unwrap());

    true
}

async fn send_to_group(msg: String) -> bool {
    let url = "http://127.0.0.1:5700/send_group_msg";
    let send_list: Vec<i64> = vec![476878378, 668128870];
    for i in send_list {
        let client = reqwest::Client::new();
        let res = match client
            .post(url)
            .json(&serde_json::json!({
                "group_id": i,
                "message": msg,
            }))
            .send()
            .await
        {
            std::result::Result::Ok(res) => res,
            Err(e) => {
                println!("Error: {}", e);
                return false;
            }
        };
        println!("{:?}", res.text().await.unwrap());
    }
    true
}
