use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use ethers::solc::artifacts::bytecode;
use mysql::prelude::*;
use mysql::*;
use regex::Regex;
use serde_json::json;
use soup::QueryBuilder;
use std::{fs, string};
use std::{thread, time};

#[derive(Debug, PartialEq, Eq)]
struct Contracts {
    address: String,
    bytecode: String,
    source_code: String,
    creation_code: String, //date
    last_tx: String,
    is_verfied: bool,
}
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    //
    //tab_source is  a tuples(address,vec!(string));
    let (username, password, ip) = read_config_file(".secret/config.json");

    let url = format!("mysql://{}:{}@{}:3306/Contracts", username, password, ip).replace("\"", ""); //remove all the " carefull if the password has one!
    println!("{}", url);
    let x = Opts::from_url(&url).unwrap();
    let pool = Pool::new(x)?;
    let mut conn = pool.get_conn()?;

    let address = "0x7451B1136b3E5D2Af1011691b95BcAA630B9E1d0";
    let bytecode = "0x41414141";
    //display_new_entries(&conn);
    from_addr_to_code_todb(address, bytecode, conn);

    Ok(())
}

fn read_config_file(path: &str) -> (String, String, String) {
    let file = fs::File::open(".secret/config.json").expect("file should open read only");
    let config: serde_json::Value =
        serde_json::from_reader(file).expect("file should be proper JSON");
    (
        config["username"].to_string(),
        config["password"].to_string(),
        config["ip"].to_string(),
    )
}

fn from_addr_to_code_todb(address: &str, bytecode: &str, mut conn: PooledConn) {
    let creation_date = "";
    let last_tx = "";
    let tab_sources: (&str, Vec<String>) = from_addr_to_source_code(address);
    if tab_sources.1.len() > 0 {
        //code is verified
        add_to_db(
            conn,
            address,
            bytecode,
            &tab_sources.1[0],
            creation_date,
            last_tx,
            true,
        );
    } else {
        //OnlyByteCode code not verified.
        add_to_db(conn, address, bytecode, "", creation_date, last_tx, false);
    }
}

fn add_to_db(
    mut conn: PooledConn,
    address: &str,
    bytecode: &str,
    sourcecode: &str,
    creation_date: &str,
    last_tx: &str,
    is_verified: bool,
) {
    // let para = params! {"Address" => address,
    // "ByteCode" => bytecode,
    // "SourceCode" => sourcecode,
    // "CreationDate" => creation_date,
    // "LastTX" => last_tx,
    // "IsVerfied" => is_verified
    // };

    let creation_date = format!(
        "{}",
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc)
    )
    .replace("UTC", "");
    let last_tx = creation_date.clone();
    let query = format!("INSERT INTO ETH (Address, ByteCode,SourceCode, CreationDate,LastTX,IsVerified) VALUES (\"{}\", \"{}\", :sourcecode, \"{}\",\"{}\",{});",address,bytecode,creation_date,last_tx,0);
    println!("Query debug :{}", query);
    let para_test = params! {"sourcecode" => sourcecode};
    let x = conn.exec_drop(query, para_test);
    println!("res of the query => {:?}", x);
}

fn display_new_entries(conn: PooledConn) {
    let mut address_last_etherscan = "".to_string();
    loop {
        let response = reqwest::blocking::get("https://etherscan.io/contractsVerified")
            .unwrap()
            .text()
            .unwrap();
        let tab = parse_tr_to_vec(&response);
        let mut i = 0;
        while address_last_etherscan != tab[i][0] {
            println!(
            "Address : {}\nContract Name : {}\nCompiler : {}\nVersion : {}\nBalance : {}\ntxns => {}\nDate Update : {}\nSource_code : {}\n-------------------------------------------------------------",
            tab[i][0], tab[i][1], tab[i][2],  tab[i][3],  tab[i][4],  tab[i][5],Utc::now().format("%d-%m-%Y %H:%M:%S"),format!("https://etherscan.io/address/{}#code",tab[i][0])
            );
            //from_addr_to_code_todb(address, bytecode, conn);
            //address_last_etherscan = tab[0][0].clone();
            if i + 1 < tab.len() {
                i += 1;
            } else {
                break;
            }
        }
        address_last_etherscan = tab[0][0].clone();
        thread::sleep(time::Duration::from_secs(5));
    }
}

fn parse_tr_to_vec(tr: &str) -> Vec<Vec<String>> {
    let mut smart_contract_tab: Vec<Vec<String>> = Vec::new();
    let regex_solidity_version = Regex::new(r"\d.\d.\d").unwrap();
    let table = table_extract::Table::find_first(tr).expect(tr);
    for row in &table {
        let address = String::from(row.get("Address").expect("Address is missing"));
        let a_selector = scraper::Selector::parse("a").unwrap();
        let document = scraper::Html::parse_document(&address);
        let address_from_selector = document.select(&a_selector).next().unwrap().inner_html();
        let contract_name = row.get("Contract Name").expect("Address is missing");
        let compiler = row.get("Compiler").expect("compiler is missing");
        let version = regex_solidity_version
            .find(row.get("Version").expect("version is missing"))
            .unwrap();
        let balance = row
            .get("Balance")
            .expect("balance is missing")
            .replace("<b>", "")
            .replace("</b>", "");
        let txns = row.get("Txns").expect("txns is missing");
        let setting = row.get("Setting").expect("setting is missing");
        let verified = row.get("Verified").expect("verified is missing");
        let source_code = "".to_string(); //get_source_code(&address_from_selector);
        smart_contract_tab.push(vec![
            address_from_selector,
            contract_name.to_string(),
            compiler.to_string(),
            version.as_str().to_string(),
            balance,
            txns.to_string(),
            setting.to_string(),
            verified.to_string(),
            source_code,
        ]);
    }
    smart_contract_tab
}
fn from_addr_to_source_code(address: &str) -> (&str, Vec<String>) {
    let mut vec_str: Vec<String> = vec![];
    let mut src = String::new();
    let link = format!("https://etherscan.io/contractdiffchecker?a1={}", address);
    let response = reqwest::blocking::get(link).unwrap().text().unwrap();
    let source_code_selector = scraper::Selector::parse("pre.sourceCode1").unwrap();
    let document = scraper::Html::parse_document(&response);
    for src_code in document.select(&source_code_selector) {
        vec_str.push(src_code.inner_html());
    }

    // let src = document
    //     .select(&source_code_selector)
    //     .next()
    //     .expect(&format!(
    //         "error cause the document look like => {:#?}",
    //         &response
    //     ))
    //     .inner_html();
    return (address, vec_str);
}

// async fn req() -> Result<(), reqwest::Error> {
//     let body = reqwest::get("https://etherscan.io/contractsVerified")
//         .await?
//         .text()
//         .await?;

//     println!("body = {:?}", body.split("").collect::<Vec<&str>>()[1]);
//     Ok(())
// }
