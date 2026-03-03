use std::io::{self, Write};
use anyhow;
use serde;

struct SummonerResponse {
    puuid : String,
    game_name : String,
    tag_line1 : String
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

    tft_api_request(summoner).await?;

    Ok(())
}


async fn tft_api_request(summoner: &str) -> anyhow::Result<()> {
    // Placeholder: this is where your pipeline goes:
    
    // 1) riot-id -> account endpoint (puuid)
    ///riot/account/v1/accounts/by-riot-id/{gameName}/{tagLine} get the api request format
    // 2) puuid -> matchlist endpoint (match ids)
    // 3) match ids -> match details (concurrently)
    // 4) aggregate “top comps”, print report

    println!("(todo) Would query Riot API for: {}", summoner);
    Ok(())
}

//Only 1 API request for the last game's comp
//8 * 10 = 80
//88 requests per run