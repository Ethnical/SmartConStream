use ethers::solc::utils::source_name;
use regex::Regex;

fn main() {
    req2();

    //tokio::runtime::Runtime::new().unwrap().block_on(req());
}

fn req2() {
    let response = reqwest::blocking::get("https://etherscan.io/contractsVerified")
        .unwrap()
        .text()
        .unwrap();

    parse_tr(&response);
    //let vec = tr.collect::<Vec<_>>();
    //println!("{:?}", vec[0]);
}

fn parse_tr(tr: &str) {
    let re = Regex::new(r"\d.\d.\d").unwrap();
    let table = table_extract::Table::find_first(tr).expect(tr);
    for row in &table {
        let address = row.get("Address").expect("Address is missing");
        let a_selector = scraper::Selector::parse("a").unwrap();
        let document = scraper::Html::parse_document(address);
        let address = document.select(&a_selector).next().unwrap().inner_html();
        let contract_name = row.get("Contract Name").expect("Address is missing");
        let compiler = row.get("Compiler").expect("compiler is missing");
        let version = re
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
        let source_code = get_source_code(&address);
        println!(
            "addr => {}\ncontract_name => {}\ncompiler =>{}\nversion => {}\nbalance => {}\ntxns => {}\nverified date => {}\nource_code =>{}\n-------------------------------------------------------------",
            address, contract_name, compiler, version.as_str(), balance, txns,verified,source_code
        );
    }

    fn get_source_code(address: &str) -> String {
        let link = format!("https://etherscan.io/contractdiffchecker?a1={}", address);
        let response = reqwest::blocking::get(link).unwrap().text().unwrap();
        let source_code_selector = scraper::Selector::parse("pre.sourceCode1").unwrap();
        let document = scraper::Html::parse_document(&response);
        let src = document
            .select(&source_code_selector)
            .next()
            .unwrap()
            .inner_html();
        String::from(src)
    }

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

// async fn req() -> Result<(), reqwest::Error> {
//     let body = reqwest::get("https://etherscan.io/contractsVerified")
//         .await?
//         .text()
//         .await?;

//     println!("body = {:?}", body.split("").collect::<Vec<&str>>()[1]);
//     Ok(())
// }
