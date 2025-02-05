use reqwest::{Client, Method, Error, header};
use serde_json::Value;

use crate::header::HeaderMap;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;

#[derive(Serialize, Deserialize)]
struct Data {
    request_id: String,
    lease_id: String,
    renewable: bool,
    lease_duration: i32,
    data:  Map<String, Value> ,
    wrap_info: Option<String>,
    warnings: Option<String>,
    auth: Option<String>,


}

async fn post_request() -> Result<(), Error> {

    let vault_url = "https://vault.ru/v1/";
    let engine = "copy_from_vault_engine";
    let client = Client::new();

    
   //get client token
    let url = vault_url.to_owned() + "auth/approle/login";
    let json_data = r#"{"role_id":"from vault","secret_id":"from vault"}"#;

   // let req = client.request(Method::from_bytes(b"LIST").unwrap(), url).body(json_data.to_owned());
   // let response = req.send().await?;

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(json_data.to_owned())
        .send()
        .await?;

    //println!("Status Code: {}", response.status());

    let response_body = response.text().await?;

    //println!("Response body: \n{}", response_body);

    let v: Value = serde_json::from_str(&response_body).unwrap();
    let client_token = (&v["auth"]["client_token"]).as_str().unwrap();
    //println!("{:?}", &client_token);

 

    //get list secret
    let url = vault_url.to_owned() + engine + "/metadata/";
    let mut headers = HeaderMap::new();
            headers.insert("X-Vault-Token", client_token.parse().unwrap());
    
    let client = reqwest::Client::builder().default_headers(headers).build()?;
    let req = client.request(Method::from_bytes(b"LIST").unwrap(), url).body(json_data.to_owned());
    let response = req.send().await?;

        //println!("Status Code: {}", response.status());
        
    let response_body = response.text().await?;

        //println!("Response body: \n{}", response_body);

    //let v: Value = serde_json::from_str(&response_body).unwrap();
    let value: Data = serde_json::from_str(&response_body).unwrap();
    let meta_map = &value.data;
    let secret_meta = &meta_map["keys"].to_string();
    let secret_meta = secret_meta.replace("\"", "");
    let secret_meta = secret_meta[1..secret_meta.len() - 1].split(",");



    //get secrets key-value
for secret in secret_meta {
    
    let secret_url = vault_url.to_owned() + engine + "/data/" + &secret;
    

    let mut headers = HeaderMap::new();
    headers.insert("X-Vault-Token", client_token.parse().unwrap());
    let client = reqwest::Client::builder().default_headers(headers).build()?;
    
    let response = client.get(&secret_url).send().await?;

    let response_body = response.text().await?;
    let v: Value = serde_json::from_str(&response_body).unwrap();
    let secret_data = &v["data"]["data"];
    let secret_data_string = secret_data.to_string();
    let secret_data_string = secret_data_string.replace(":", ";");
    let secret_data_string = &secret_data_string[1..secret_data_string.len() - 1];

    
    if secret_data.is_null() {
        println!("{} - not contains secret!", &secret_url);
    } else {
        println!("{};{}", &secret, &secret_data_string);
    }    

    


    }   



/*

//get secrets key-value
let url = "https://vault.ru/v1/engine/data/test_secret";



let mut headers = HeaderMap::new();
    headers.insert("X-Vault-Token", client_token.parse().unwrap());

    let client = reqwest::Client::builder().default_headers(headers).build()?;


let response = client
    .get(url)
    .send()
    .await?;

let response_body = response.text().await?;

let v: Value = serde_json::from_str(&response_body).unwrap();
let secret_data = &v["data"]["data"];

println!("{}", &secret_data);

*/

    Ok(())

}

#[tokio::main]
async fn main() -> Result<(), Error> {

    post_request().await?;
    Ok(())
}
