use std::io::{self, Write};
use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    api_key: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SummonerResponse {
    puuid : String,
    game_name : String,
    tag_line : String
}

#[derive(Debug, Deserialize)]
struct MatchResponse {
    metadata: MatchMetadata,
    info: MatchInfo,
}

#[derive(Debug, Deserialize)]
struct MatchMetadata {
    match_id: String,
    participants: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MatchInfo {
    participants: Vec<Participant>,
}

#[derive(Debug, Deserialize)]
struct Participant {
    puuid: String,
    placement: u32,
    traits: Vec<TftTrait>,
    units: Vec<TftUnit>,
}

#[derive(Debug, Deserialize)]
struct TftTrait {
    name: String,
    num_units: u32,
    style: u32,        // 0 = inactive, 1 = bronze, 2 = silver, 3 = gold, 4 = chromatic
    tier_current: u32,
    tier_total: u32,
}

#[derive(Debug, Deserialize)]
struct TftUnit {
    character_id: String,
    #[serde(rename = "itemNames")]
    item_names: Vec<String>,
    rarity: u32,
    tier: u32,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    print!("\x1B[2J\x1B[1;1H"); // clear screen
    print!("Enter a valid Riot Summoner Name with ID (Ex: Rusty#NA1): ");
    io::stdout().flush().unwrap();

    let mut __summoner_name__ = String::new();
    io::stdin().read_line(&mut __summoner_name__).expect("Failed to read line");
    let summoner = __summoner_name__.trim();
    println!("Getting a detailed report for summoner: {}", summoner);

    let puuid: String = tft_summoner_request(summoner).await?;
    let matches: Vec<String> = tft_matchid_request(&puuid).await?;
    
    println!("Found {} matches", matches.len());

    let comp = get_last_played_comp(&matches[0], &puuid).await?;

    println!("{} played {} last game.", summoner, comp);


    Ok(())
}

async fn tft_summoner_request(summoner: &str) -> anyhow::Result<String> {
    // Placeholder: this is where your pipeline goes:
    // 1) riot-id -> account endpoint (puuid)
    // /riot/account/v1/accounts/by-riot-id/{gameName}/{tagLine} get the api request format
    let (game_name, tag_line) = summoner.split_once("#").ok_or_else(|| anyhow::anyhow!("Invalid format. Use SummonerName#Tag"))?;
    
    let config : Config = serde_yaml::from_str(
        &std::fs::read_to_string("config/environmental.yaml").with_context(|| "YAML file failed")?
    )?;

    let url = format!(
        "https://americas.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
        game_name, tag_line
    );

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("X-Riot-Token", config.api_key)
        .send()
        .await?
        .error_for_status()?
        .json::<SummonerResponse>()
        .await?;


    // println!("PUUID: {}", res.puuid);
    println!("Name: {}#{}", res.game_name, res.tag_line);
    // 2) puuid -> matchlist endpoint (match ids)
    // 3) match ids -> match details (concurrently)
    // 4) aggregate “top comps”, print report

    println!("Valid Summoner: {}", summoner);
    let puuid = res.puuid;
    Ok(puuid)    
}

async fn tft_matchid_request(puuid: &str) -> anyhow::Result<Vec<String>> {
    let config : Config = serde_yaml::from_str(
        &std::fs::read_to_string("config/environmental.yaml").with_context(|| "YAML file failed")?
    )?;
    
    let start_count = 0;
    let end_count = 10;

    let url = format!(
        "https://americas.api.riotgames.com/tft/match/v1/matches/by-puuid/{}/ids?start={}&count={}",
        puuid, start_count, end_count
    );

    let client = reqwest::Client::new();
    let res = client
            .get(&url)
            .header("X-Riot-Token", config.api_key)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<String>>()
            .await?;

    Ok(res)
}

async fn get_last_played_comp(matchid: &str, puuid: &str) -> anyhow::Result<(String)> {
    let config : Config = serde_yaml::from_str(
        &std::fs::read_to_string("config/environmental.yaml").with_context(|| "YAML file failed")?
    )?;

    let url = format!(
        "https://americas.api.riotgames.com/tft/match/v1/matches/{}",
        matchid,
    );

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("X-Riot-Token", config.api_key)
        .send()
        .await?
        .error_for_status()?
        .json::<MatchResponse>()
        .await?;

    //parse res
    //Find traits (Top 2 traits) + find the tier 3, or tier 2 that has items
    let personal_summoner = res.info.participants
        .iter()
        .find(|p| p.puuid == puuid)
        .ok_or_else(|| anyhow::anyhow!("PUUID not found in match"))?;

    let mut active_traits: Vec<&TftTrait> = personal_summoner.traits
        .iter()
        .filter(|t| t.style > 0)
        .collect();

    active_traits.sort_by(|a, b| b.num_units.cmp(&a.num_units));
    // If the top trait has 5+ units, return just that one
    // Otherwise return the top 2
    let comp_name = if let Some(top) = active_traits.first() {
        if top.num_units > 5 {
            clean_trait_name(&top.name)
        } else {
            let names: Vec<String> = active_traits
                .iter()
                .take(2)
                .map(|t| clean_trait_name(&t.name))
                .collect();
            names.join(" + ")
        }
    } else {
        "Unknown Comp".to_string()
    };

    Ok(comp_name)
}

fn clean_trait_name(name: &str) -> String {
    name.split('_').last().unwrap_or(name).to_string()
}
//Only 1 API request for the last game's comp
//8 * 10 = 80
//88 requests per run



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_summoner_request() {
        let puuid = tft_summoner_request("Aidan B Pianist#NA1");
        assert!(puuid.await.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_format() {
        let result = tft_summoner_request("BadExample");
        assert!(result.await.is_err());
    }
}