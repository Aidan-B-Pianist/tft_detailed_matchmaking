use std::io::{self, Write};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]

struct SummonerResponse {
    puuid : String,
    game_name : String,
    tag_line : String
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    print!("\x1B[2J\x1B[1;1H"); // clear screen
    print!("Enter a valid Riot Summoner Name with ID (Ex: Rusty#NA1): ");
    io::stdout().flush().unwrap(); // ensures the prompt shows before input
    
    let mut __summoner_name__ = String::new();
    io::stdin().read_line(&mut __summoner_name__).expect("Failed to read line");
    let summoner = __summoner_name__.trim();
    println!("Getting a detailed report for summoner: {}", summoner);

    tft_summoner_request(summoner).await?;
    // tft_matchid_request(res).await?;

    Ok(())
}


async fn tft_summoner_request(summoner: &str) -> anyhow::Result<()> {
    // Placeholder: this is where your pipeline goes:
    // 1) riot-id -> account endpoint (puuid)
    // /riot/account/v1/accounts/by-riot-id/{gameName}/{tagLine} get the api request format
    let (game_name, tag_line) = summoner.split_once("#").ok_or_else(|| anyhow::anyhow!("Invalid format. Use SummonerName#Tag"))?;

    let config = std::fs::read_to_string("environmental.yaml")?;

    let url = format!(
        "https://americas.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
        game_name, tag_line
    );

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("X-Riot-Token", config)
        .send()
        .await?
        .error_for_status()?
        .json::<SummonerResponse>()
        .await?;

    println!("PUUID: {}", res.puuid);
    println!("Name: {}#{}", res.game_name, res.tag_line);
    // 2) puuid -> matchlist endpoint (match ids)
    // 3) match ids -> match details (concurrently)
    // 4) aggregate “top comps”, print report

    println!("(todo) Would query Riot API for: {}", summoner);
    Ok(())
}

async fn tft_matchid_request(summoner: &str) -> anyhow::Result<()> {
    

    Ok(())
}

//Only 1 API request for the last game's comp
//8 * 10 = 80
//88 requests per run