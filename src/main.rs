extern crate xmlrpc;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use xmlrpc::{Request, Value};
use serde::{Serialize, Deserialize};
use std::io::prelude::*;
use std::fs::File;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SumaInfo {
    hostname: String,
    user_name: String,
    password: String,
}

#[derive(Deserialize)]
pub struct GetServerId {
   hostname: String,
}

impl SumaInfo {
    fn new(file: &String) -> SumaInfo {
        let mut f = File::open(file).expect("Could not read file");
        let mut buffer = String::new();

        f.read_to_string(&mut buffer).expect("failed to read file into buffer as string.");
        let deserialized_map: SumaInfo = match serde_yaml::from_str(&buffer) {
            Ok(i) => i,
            Err(_) => panic!("getting yaml failed.")
        };
        return deserialized_map
    }
}

fn login(s: &SumaInfo) -> String {
    let suma_request = Request::new("auth.login").arg(String::from(&s.user_name)).arg(String::from(&s.password)); 
    let request_result = suma_request.call_url(String::from(&s.hostname));
    match &request_result {
        Err(e) => {
            println!("Could not login to SUMA server. {}", e);
            std::process::exit(1);
        },
        Ok(i) => match i.as_str() {
            
            Some(q) => return q.to_string(),
            None => std::process::exit(1),
        }
    }
}

fn logout(k: &String, s: &SumaInfo) -> i32 {
    let suma_logout_request = Request::new("auth.logout").arg(k.to_string());
    let suma_logout_result = suma_logout_request.call_url(String::from(&s.hostname));
    match &suma_logout_result {
        Err(e) => {
            println!("Could not logout. {}", e);
            std::process::exit(1);
        },
        Ok(i) => match i.as_i32() {
            Some(q) => return q,
            None => std::process::exit(1),
        }
    }
}

fn get_systemid(key: &String, s: &String, z: &SumaInfo) -> Result<i32, &'static str> {

    let get_system_id = Request::new("system.getId").arg(String::from(key)).arg(s.to_string());
    let get_system_id_result = get_system_id.call_url(String::from(&z.hostname));

    match get_system_id_result.unwrap().as_array() {
        Some(i) => {
            if i.len() > 0 {
                match i[0].as_struct() {
                    Some(h) => match h[&"id".to_string()].as_i32() {
                        Some(j) => return Ok(j),
                        None => Err("invalid server id, no integer found."),
                    }
                    None => Err("invalid server id, no struct found."),
                }
            } else {
                Err("invalid server id in array.")
            }
        },
        None => Err("invalid server id, no array."),
    }
}

fn get_system_details(key: &String, s: i32, z: &SumaInfo) -> Result<xmlrpc::Value, &'static str> {

    let get_system_details = Request::new("system.getDetails").arg(String::from(key)).arg(s);
    let get_system_details_result = get_system_details.call_url(String::from(&z.hostname));

    match get_system_details_result {
        Ok(i) => Ok(i),
        Err(_) => Err("invalid server details."),
    }
}

fn get_system_details_html(x: Value) -> String {
    //println!("{}", x.as_struct().unwrap().get("minion_id").unwrap().as_str().unwrap());
    let system_details_fields = vec!["minion_id", "machine_id", "base_entitlement", "virtualization", "contact_method"];

    let mut body = String::new();
    for s in system_details_fields {
        body.push_str(&("<p>".to_owned() + s + ": "));
        let myval = match x.as_struct() {
            Some(i) => i.get(s).unwrap().as_str().unwrap().to_string(),
            None => "Not found".to_string(),
        };
        body.push_str(&myval);
        body.push_str("</p>");
    }
    
    return body
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello BoJin!")
}

#[get("/getid")]
async fn getid(web::Query(info): web::Query<GetServerId>) -> impl Responder {

    let mut suma_info: SumaInfo = SumaInfo::new(&String::from("test.yaml"));
    suma_info.hostname.insert_str(0, "http://");
    suma_info.hostname.push_str("/rpc/api");
    println!("suma host api url: {:?}", &suma_info.hostname);

    let key = login(&suma_info);
            
    let systems_id = get_systemid(&key, &info.hostname, &suma_info);
    //println!("systemdi: {:?}", systems_id.unwrap());
    let sid = match systems_id {
        Ok(i) => i,
        Err(s) => return HttpResponse::Ok().body(&String::from(s)),
    };
    let system_details = get_system_details(&key, sid, &suma_info);
    println!("Logout successful - {}", logout(&key, &suma_info));
    let system_details_html_body = get_system_details_html(system_details.unwrap());
    
    return HttpResponse::Ok().body(&String::from(system_details_html_body))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey Bo!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(getid)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind("127.0.0.1:8888")?
    .run()
    .await
}
