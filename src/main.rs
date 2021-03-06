extern crate xmlrpc;
extern crate clap;

#[macro_use]
extern crate log;

use env_logger::Env;

use clap::{Arg, App};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_web_httpauth::extractors::basic::{BasicAuth};
use actix_web::{dev::ServiceRequest, get, web, App as OtherApp, HttpResponse, HttpServer, Responder, middleware, error, Result};
use actix_web_httpauth::middleware::HttpAuthentication;
use xmlrpc::{Request, Value};
use serde::{Serialize, Deserialize};
use std::io::prelude::*;
use std::fs::File;
use once_cell::sync::OnceCell;
use derive_more::{Display, Error};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SumaInfo {
    hostname: String,
    user_name: String,
    password: String,
    certificate: String,
    tls_key: String,
    restapi_port: i32,
    http_basic_auth_user: String,
    http_basic_auth_password: String,
}

#[derive(Deserialize)]
pub struct GetServerId {
   hostname: String,
}

impl SumaInfo {

    pub fn global() -> &'static SumaInfo {
        INSTANCE.get().expect("SumaInfo is not initialized")
    }

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

static INSTANCE: OnceCell<SumaInfo> = OnceCell::new();

#[derive(Debug, Display, Error)]
#[display(fmt = "suma-rest-api error: {}", message)]
struct MyError {
    message: &'static str,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for MyError {}

fn login() -> String {
    let s = SumaInfo::global();
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

fn logout(k: &String) -> i32 {
    let s = SumaInfo::global();
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

fn get_systemid(key: &String, s: &String) -> Result<i32, &'static str> {

    let z = SumaInfo::global();
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

fn get_system_details(key: &String, s: i32) -> Result<xmlrpc::Value, &'static str> {

    let z = SumaInfo::global();
    let get_system_details = Request::new("system.getDetails").arg(String::from(key)).arg(s);
    let get_system_details_result = get_system_details.call_url(String::from(&z.hostname));

    match get_system_details_result {
        Ok(i) => Ok(i),
        Err(_) => Err("invalid server details."),
    }
}

fn get_errata_list(key: &String, s: i32) -> Result<Vec<i32>, &'static str> {

    let z = SumaInfo::global();
    let mut patchlist: Vec<i32> = vec![];
    let get_errata_list = Request::new("system.getRelevantErrata").arg(String::from(key)).arg(s);
    let errata_result = get_errata_list.call_url(String::from(&z.hostname));
    match errata_result {
        Ok(i) => {
            if i.as_array().unwrap().len() > 0 {
                i.as_array().unwrap().into_iter().for_each(|x| {
                    let id = x.as_struct().unwrap().get("id").unwrap().as_i32().unwrap();
                    patchlist.push(id);
                });
            }
            Ok(patchlist)
        },
        Err(_) => Err("No patch found."),
    }
}

fn patch_schedule(key: &String, s: i32, erratalist: Vec<i32>) -> Result<i32, xmlrpc::Error> {

    let z = SumaInfo::global();
    let mut value_id_list: Vec<Value> = Vec::new();
    for s in &erratalist {
        value_id_list.push(Value::Int(*s));
    }
    let patch_job = Request::new("system.scheduleApplyErrata").arg(String::from(key)).arg(s).arg(Value::Array(value_id_list));
    let patch_job_id = patch_job.call_url(String::from(&z.hostname));
    //println!("jobid {:?}", &patch_job_id.as_ref().unwrap());
    match patch_job_id {
        Ok(s) => Ok(s.as_array().unwrap()[0].as_i32().unwrap()),
        Err(e) => Err(e),
    }
}

fn get_system_details_html(x: Value) -> String {

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
    let html = String::from("<a href=https://github.com/bjin01/rust-suma-api title=SUMA REST API>Go here for How-To-Use</a>");
    HttpResponse::Ok().body("<h1>This is SUMA REST API</h1>".to_owned() + &html)
}

#[get("/getinfo")]
async fn getinfo(web::Query(info): web::Query<GetServerId>) -> impl Responder {
   
    let key = login();
            
    let systems_id = get_systemid(&key, &info.hostname);
    //println!("systemdi: {:?}", systems_id.unwrap());
    let sid = match systems_id {
        Ok(i) => i,
        Err(s) => return HttpResponse::Ok().body(&String::from(s)),
    };
    let system_details = get_system_details(&key, sid);
    println!("Logout successful - {}", logout(&key));
    let system_details_html_body = get_system_details_html(system_details.unwrap());
    
    return HttpResponse::Ok().body(&String::from(system_details_html_body))
}

//#[get("/patch")]
async fn patch(web::Query(info): web::Query<GetServerId>) -> impl Responder {
    let key = login();

    let systems_id = get_systemid(&key, &info.hostname);
    //println!("systemdi: {:?}", systems_id.unwrap());
    let sid = match systems_id {
        Ok(i) => i,
        Err(s) => return HttpResponse::Ok().body(&String::from(s)),
    };
    let get_errata_list_result = get_errata_list(&key, sid);
    let errata_list = match get_errata_list_result {
        Ok(i) => i,
        Err(s) => return HttpResponse::Ok().body(&String::from(s)),
    };
    
    let patch_job_result = patch_schedule(&key, sid, errata_list);
    println!("Logout successful - {}", logout(&key));
    match patch_job_result {
        Ok(i) => return HttpResponse::Ok().body(&String::from("Jobid: ".to_owned() + &i.to_string())),
        Err(s) => return HttpResponse::Ok().body(&String::from(s.to_string())),
    };
}

async fn suma(s: String) -> impl Responder {
    HttpResponse::Ok().body(s)
}

async fn validator(req: ServiceRequest, credentials: BasicAuth) -> Result<ServiceRequest, actix_web::Error> {
    
    let local_suma_info = SumaInfo::global();
    info!("validating basic authentication...");

    if credentials.user_id().eq(&local_suma_info.http_basic_auth_user) && credentials.password().unwrap().eq(&local_suma_info.http_basic_auth_password) {
        Ok(req)
    } else {
        let myerr = MyError {
            message: "login failed."
        };
        Err(actix_web::Error::from(myerr))
    } 
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let matches = App::new("SUSE Manager - rest api")
        .version("0.1.0")
        .author("Bo Jin <bo.jin@suse.com>")
        .about("patch systems by calling rest api")
        .arg(Arg::with_name("config")
                 .short("c")
                 .long("config")
                 .takes_value(true)
                 .help("yaml config file with login credentials"))
        .get_matches();
    let yaml_file = matches.value_of("config").unwrap_or("test.yaml");
    let mut suma_info: SumaInfo = SumaInfo::new(&String::from(yaml_file));
    suma_info.hostname.insert_str(0, "http://");
    suma_info.hostname.push_str("/rpc/api");

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(&suma_info.tls_key, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(&suma_info.certificate).unwrap();

    let server_port = suma_info.restapi_port;
    
    INSTANCE.set(suma_info).expect("set suma_info as static var failed.");

    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(validator);

        OtherApp::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Logger::new("%a %{User-Agent}"))
            .wrap(auth)
            .service(hello)
            .service(getinfo)
            .route("/patch", web::get().to(patch))
            .route("/suma", web::get().to(|| suma("ok".to_string())))
    })
    .bind_openssl("0.0.0.0:".to_owned() + &server_port.to_string(), builder)?
    .run()
    .await

}
