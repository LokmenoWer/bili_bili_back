use std::io::Read;

use serde::ser::SerializeStruct;


use reqwest::{Client};


pub struct bilib_login {
    pub  url:String,
    pub  qrcode_key:String,
}
pub struct bilib_cookie {
    pub  DedeUserID:String,
    pub  DedeUserID__ckMd5:String,
    pub  Expires:String,
    pub SESSDATA:String,
    pub bili_jct:String,
}
//为bilib_cookie实现serde::ser::Serialize
impl serde::ser::Serialize for bilib_cookie {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer
    {
        let mut state = serializer.serialize_struct("bilib_cookie", 5)?;
        state.serialize_field("DedeUserID", &self.DedeUserID)?;
        state.serialize_field("DedeUserID__ckMd5", &self.DedeUserID__ckMd5)?;
        state.serialize_field("Expires", &self.Expires)?;
        state.serialize_field("SESSDATA", &self.SESSDATA)?;
        state.serialize_field("bili_jct", &self.bili_jct)?;
        state.end()
    }
}


pub async fn get_login() -> bilib_login{
    //访问https://passport.bilibili.com/x/passport-login/web/qrcode/generate
    //获取二维码key
    let client = Client::new();
    let resp = match client.get("https://passport.bilibili.com/x/passport-login/web/qrcode/generate").send().await {
        Ok(resp) => resp.text().await.unwrap(),
        Err(_) => panic!("error"),
    };
    let json: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let qrcode_key = json["data"]["qrcode_key"].to_string();
    let qrcode_key = qrcode_key.trim_matches('"').to_string();
    let url = json["data"]["url"].to_string();
    let url = url.trim_matches('"').to_string();
    let login = bilib_login{
        url:url,
        qrcode_key:qrcode_key,
    };
    login
}
//重写方法，接收bilib_login结构体
pub async fn check_login(login:& bilib_login) -> (bool,String){
    //访问https://passport.bilibili.com/x/passport-login/web/qrcode/poll
    //检查是否登录
    let client = Client::new();
    let resp = match client.get("https://passport.bilibili.com/x/passport-login/web/qrcode/poll?").query(&[("qrcode_key", &login.qrcode_key)]).send().await {
        Ok(resp) => resp.text().await.unwrap(),
        Err(_) => panic!("error"),
    };
    let json: serde_json::Value = serde_json::from_str(&resp).unwrap();
    let status = json["data"]["code"].to_string();
    let status = status.trim_matches('"').to_string();
   match status.as_str() {
       "86101" => {
              println!("未扫码");
              return(false,"".to_string());
            }
       "86090" => {
              println!("已扫码未登录");
              return(false,"".to_string());
            }
       "0" =>  {
        println!("登录成功");
        //打印url
        println!("{}",json["data"]["url"].to_string());    
        
        
        //返回true和bilib_cookie结构体
        let url = json["data"]["url"].to_string();
        let url = url.trim_matches('"').to_string();
        return (true,url);
      }
       _ => panic!("error"),
   }
       
   
}

//为bilib_login实现clone
impl Clone for bilib_login {
    fn clone(&self) -> Self {
        bilib_login {
            url: self.url.clone(),
            qrcode_key: self.qrcode_key.clone(),
        }
    }
}

//为bilib_cookie实现clone
impl Clone for bilib_cookie {
    fn clone(&self) -> Self {
        bilib_cookie {
            DedeUserID: self.DedeUserID.clone(),
            DedeUserID__ckMd5: self.DedeUserID__ckMd5.clone(),
            Expires: self.Expires.clone(),
            SESSDATA: self.SESSDATA.clone(),
            bili_jct: self.bili_jct.clone(),
        }
    }
}

pub fn read_cookie() -> (bilib_cookie,bool){
    //读取cookie
    let mut cookie = bilib_cookie{
        DedeUserID:"".to_string(),
        DedeUserID__ckMd5:"".to_string(),
        Expires:"".to_string(),
        SESSDATA:"".to_string(),
        bili_jct:"".to_string(),
    };
    let mut flag = false;
    let mut file = match std::fs::File::open("cookie.txt") {
        Ok(file) => file,
        Err(_) => {
            println!("未找到cookie.txt");
            return (cookie,flag);
        }
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {
           //按json格式解析cookie
            let json: serde_json::Value = serde_json::from_str(&contents).unwrap();
            cookie.DedeUserID = json["DedeUserID"].to_string();
            cookie.DedeUserID__ckMd5 = json["DedeUserID__ckMd5"].to_string();
            cookie.Expires = json["Expires"].to_string();
            cookie.SESSDATA = json["SESSDATA"].to_string();
            cookie.bili_jct = json["bili_jct"].to_string();
            //判断所有cookie是否不为空
            if cookie.DedeUserID == "" || cookie.DedeUserID__ckMd5 == "" || cookie.Expires == "" || cookie.SESSDATA == "" || cookie.bili_jct == "" {
                println!("cookie.txt格式错误");
                return (cookie,flag);
            }
            flag = true;
            return (cookie,flag);
        }
        Err(_) => {
            println!("error");
            return (cookie,flag);
        }
    }
}