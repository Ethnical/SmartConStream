use regex::Regex;
use std::{f32::consts::E, thread, time};

fn main() {
    req2();

    //tokio::runtime::Runtime::new().unwrap().block_on(req());
}

fn req2() {
    let mut address_last_etherscan = "".to_string();
    while (true) {
        let response = reqwest::blocking::get("https://etherscan.io/contractsVerified")
            .unwrap()
            .text()
            .unwrap();
        let mut tab = parse_tr_to_vec(&response);
        let mut i = 0;
        while (address_last_etherscan != tab[i][0]) {
            println!(
            "Address : {}\nContract Name : {}\nCompiler : {}\nVersion : {}\nBalance : {}\ntxns => {}\nVerified date : {}\nSource_code : {}\n-------------------------------------------------------------",
            tab[i][0], tab[i][1], tab[i][2],  tab[i][3],  tab[i][4],  tab[i][5],"N/A",format!("https://etherscan.io/address/{}#code",tab[i][0])
            );

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

    //let vec = tr.collect::<Vec<_>>();
    //println!("{:?}", vec[0]);
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

    /*
    let document = scraper::Html::parse_document(html);

    let a_selector = scraper::Selector::parse("a").unwrap();
    let address = document.select(&a_selector).next().unwrap();
    let td: Vec<&str> = html.split("td>").collect();

    //println!("{:?}", re.find(html).unwrap().end());
    let soup = Soup::new(html);
    let b = soup.tag("td").find().expect("Couldn't find tag 'b'");
    */
    //let x: Vec<&str> = tr.split("data-toggle=\"tooltip\">").collect();
    //println!("{:?}", x.display());
    // println()
}
fn get_source_code(address: &str) -> String {
    let link = format!("https://etherscan.io/contractdiffchecker?a1={}", address);
    let response = reqwest::blocking::get(link).unwrap().text().unwrap();
    let source_code_selector = scraper::Selector::parse("pre.sourceCode1").unwrap();
    let document = scraper::Html::parse_document(&response);
    let src = document
        .select(&source_code_selector)
        .next()
        .expect(&format!(
            "error cause the document look like => {:#?}",
            &response
        ))
        .inner_html();
    String::from(src)
}

// async fn req() -> Result<(), reqwest::Error> {
//     let body = reqwest::get("https://etherscan.io/contractsVerified")
//         .await?
//         .text()
//         .await?;

//     println!("body = {:?}", body.split("").collect::<Vec<&str>>()[1]);
//     Ok(())
// }
