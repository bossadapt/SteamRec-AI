use dotenv::dotenv;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::serde::json::{serde_json, Json};
use rocket::tokio::fs::File;
use rocket::tokio::time::sleep;
use rocket::{tokio, Request, Response, State};
use rusqlite::Connection;
use scraper::Html;
use serde::{Deserialize, Serialize};
use std::env::var;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Duration;
#[macro_use]
extern crate rocket;
extern crate reqwest;

#[derive(Debug)]
struct Visability {
    games: bool,
    reviews: bool,
}
/// API CALL FOR GETTING INITIAL CLASSIFICATION DATA ///
/// #[derive(Deserialize)]

#[derive(Deserialize)]
struct GameInfoData {
    data: GameInfo,
}
#[derive(Clone, Deserialize, Serialize)]
struct GameInfo {
    name: String,
    steam_appid: u32,
    #[serde(default)]
    score: f32,
    is_free: bool,
    short_description: String,
    developers: Option<Vec<String>>,
    header_image: String,
    release_date: ReleaseDate,
    platforms: Platforms,
    price_overview: Option<PriceOverview>,
    content_descriptors: ContentDescriptors,
}
#[derive(Clone, Deserialize, Serialize)]
struct PriceOverview {
    final_formatted: String,
}
#[derive(Clone, Deserialize, Serialize)]
struct ContentDescriptors {
    ids: Vec<u8>,
    notes: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
struct ReleaseDate {
    coming_soon: bool,
    date: String,
}
#[derive(Clone, Deserialize, Serialize)]
struct Platforms {
    windows: bool,
    mac: bool,
    linux: bool,
}
// TYPES FOR API AND SCRAPER during use////////////
#[derive(Debug, Deserialize)]
struct GameListData {
    response: GameRequest,
}
#[derive(Debug, Deserialize)]
struct GameRequest {
    #[serde(default)]
    games: Vec<Game>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Game {
    appid: u32,
    #[serde(default)]
    playtime_2weeks: u16,
    #[serde(default)]
    playtime_forever: u32,
    #[serde(default)]
    is_recommended: i8,
}

#[derive(Debug)]
struct Review {
    game_id: u32,
    is_recommended: bool,
    time_played: u32,
}
///////////response structs////////////////
#[derive(Serialize)]
struct Data {
    success: bool,
    error: String,
    games_included: bool,
    reviews_included: bool,
    games: Vec<GameInfo>,
}
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Prediction Response",
            kind: Kind::Response,
        }
    }
    ////////////////////////////////////////////////
    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

fn error_out(msg: &str) -> Json<Data> {
    Json(Data {
        success: false,
        error: msg.to_owned(),
        games_included: false,
        reviews_included: false,
        games: vec![],
    })
}
#[get("/convert/<type_given>/<id>")]
async fn convert_link(type_given: &str, id: &str, state: &State<IndividualState>) -> Json<Data> {
    let mut games_included = false;
    let mut reviews_included = false;
    println!("Got request for /{}/{}", type_given, id);
    let visability_check = get_visibility(type_given, id, &state.client).await;
    if let Ok((visability, steam_id)) = visability_check {
        if visability.games || visability.reviews {
            let mut games: Vec<Game> = vec![];
            let mut reviews: Vec<Review> = vec![];
            if visability.games {
                if let Ok(new_games) = get_game_list(&steam_id, &state.steam_api).await {
                    if new_games.len() > 0 {
                        games_included = true;
                        games = new_games;
                    }
                }
            }
            if visability.reviews {
                if let Ok(new_reviews) = get_review_list(&steam_id, &state.client).await {
                    if new_reviews.len() > 0 {
                        reviews_included = true;
                        reviews = new_reviews;
                    }
                }
            }
            let mut scorelist = games_and_reviews_into_scorelist(
                combine_games_and_reviews(games, reviews),
                &state.classifications,
            );
            return Json(Data {
                success: true,
                error: "".to_owned(),
                games_included: games_included,
                reviews_included: reviews_included,
                games: get_ai_guess(scorelist.0, &mut scorelist.1, &state.classifications).await,
            });
        } else {
            return error_out("Neither game nor reviews are public");
        }
    } else {
        return error_out(&visability_check.unwrap_err());
    }
}
async fn get_ai_guess(
    scores: Vec<f32>,
    already_owned: &mut Vec<usize>,
    classifications: &Vec<GameInfo>,
) -> Vec<GameInfo> {
    let req = format!(
        "http://127.0.0.1:5002/predict/{}",
        build_request_string_from_array(scores)
    );
    let mut python_predict = reqwest::get(req).await.unwrap().text().await.unwrap();
    //remove "[" and "]"
    python_predict = python_predict[1..python_predict.len() - 2].to_string();
    let python_predict_vec: Vec<f32> = python_predict
        .trim()
        .split(",")
        .map(|pred| {
            let output = pred.trim().parse().unwrap();
            output
        })
        .collect();
    already_owned.reverse();
    let mut new_gamelist: Vec<GameInfo> = Vec::new();
    for x in 0..classifications.len() {
        if already_owned.is_empty() || x != already_owned[already_owned.len() - 1] {
            let mut current = classifications[x].clone();
            current.score = python_predict_vec[x];
            new_gamelist.push(current);
        } else {
            already_owned.pop();
        }
    }
    new_gamelist
}
async fn get_raw_page_holding_state(
    url: &String,
    time_to_sleep: u64,
    client: &reqwest::Client,
) -> Result<String, reqwest::Error> {
    sleep(Duration::from_secs(time_to_sleep)).await;
    return client.get(url).send().await?.text().await;
}
async fn get_raw_page(url: &String, time_to_sleep: u64) -> Result<String, String> {
    sleep(Duration::from_secs(time_to_sleep)).await;
    match reqwest::get(url).await {
        Ok(resp) => return Ok(resp.text().await.unwrap()),
        Err(_err) => {
            sleep(Duration::from_secs(60)).await;
            return match reqwest::get(url).await {
                Ok(resp) => return Ok(resp.text().await.unwrap()),
                Err(_err) => return Err(format!("Failed to grab raw webpage at: {}", url)),
            };
        }
    }
}
struct Output {
    app_id: String,
}
fn remove_dynamic_start(initial_json: String) -> String {
    let mut index = 0;
    let initial_trimed = initial_json.trim();
    let mut initial_chars = initial_trimed.chars();
    let mut parenthesis_count = 0;
    for x in 0..initial_trimed.len() {
        if initial_chars.next().unwrap_or('{' as char) == '{' as char {
            parenthesis_count += 1;
            if parenthesis_count == 2 {
                index = x;
                break;
            }
        }
    }
    if index == 0 {
        return "".to_owned();
    }
    let output = initial_trimed[index..initial_json.len() - 1].to_owned();
    return output;
}
async fn get_classification_game_list() -> Vec<GameInfo> {
    let mut detailed_classification_list: Vec<GameInfo> = Vec::new();
    let file_path = "./detailedClassifications.json";
    if !(Path::new(file_path).exists()) {
        println!("classification list not found, attempting to reach sql to build csv");
        let con = Connection::open("D:\\data\\reformedUserData").unwrap();
        let mut stmt = con.prepare("SELECT gameID FROM classifications").unwrap();
        let output_list = stmt
            .query_map([], |row| {
                Ok(Output {
                    app_id: row.get(0).unwrap(),
                })
            })
            .unwrap();
        let mut count = 0;
        for id in output_list {
            count += 1;
            let current_id = id.unwrap().app_id;
            let current_link: String = format!(
                "https://store.steampowered.com/api/appdetails?appids={}",
                current_id
            );
            let game_json_string = get_raw_page(&current_link, 2).await.unwrap();
            let modified_json_string = remove_dynamic_start(game_json_string);
            let game_info: GameInfoData = match serde_json::from_str(&modified_json_string) {
                Ok(result) => result,
                Err(er) => {
                    println!("Error deserializing: {}", er);
                    let game_json_string_2 = get_raw_page(&current_link, 2).await.unwrap();
                    let modified_json_string_2 = remove_dynamic_start(game_json_string_2);
                    serde_json::from_str(&modified_json_string_2).unwrap_or(GameInfoData {
                        data: GameInfo {
                            name: "unknown".to_owned(),
                            steam_appid: current_id.parse::<u32>().unwrap(),
                            score: 0.0,
                            short_description: "".to_owned(),
                            developers: Some(vec![]),
                            header_image: "".to_owned(),
                            release_date: ReleaseDate {
                                coming_soon: false,
                                date: "".to_owned(),
                            },
                            platforms: Platforms {
                                windows: false,
                                mac: false,
                                linux: false,
                            },
                            content_descriptors: ContentDescriptors {
                                ids: vec![],
                                notes: Some("".to_owned()),
                            },
                            price_overview: Some(PriceOverview {
                                final_formatted: "".to_owned(),
                            }),
                            is_free: false,
                        },
                    })
                }
            };
            println!(
                "adding classification #{}, Name: {}",
                count,
                game_info.data.name.to_owned()
            );
            detailed_classification_list.push(game_info.data);
        }
        let file = File::create(file_path).await.unwrap();
        println!("writing json classification list");
        let mut writer = BufWriter::new(file.try_into_std().unwrap());
        serde_json::to_writer(&mut writer, &detailed_classification_list).unwrap();
        writer.flush().unwrap();
    } else {
        println!("found existing classifications");
        let file = File::open(file_path).await.unwrap();
        detailed_classification_list = serde_json::from_reader(file.into_std().await).unwrap();
    }

    detailed_classification_list
}

fn get_steam_id_from_profile(document: &Html) -> Result<String, String> {
    let ideal_part: &str = "\"steamid\":\"";
    let scraper_selector =
        scraper::Selector::parse("div.responsive_page_template_content").unwrap();
    let comment_section = document.select(&scraper_selector);
    if let Some(script_scraped) = comment_section.into_iter().nth(0) {
        let current_html: String = script_scraped.html();
        if let Some(id_idx) = current_html.find(ideal_part) {
            let start = id_idx + 11;
            return Ok(current_html[start..start + 17].to_string());
        } else {
            return Err("Custom url that does not exist".to_owned());
        }
    } else {
        return Err("Custom url that does not exist".to_owned());
    }
}
//proper link will be checked if

//1. visit profile and ensure it exists
//2. ensure games and reviews visability

/// returns what whether it can see reviews and games and the steam ID
async fn get_visibility(
    type_id: &str,
    id: &str,
    client: &reqwest::Client,
) -> Result<(Visability, String), String> {
    //check that freinds, games and reviews are visable via their home profile
    let url = format!("https://steamcommunity.com/{}/{}", type_id, id);
    let get_attempt = get_raw_page_holding_state(&url, 1, client).await;
    let raw_webpage: String;
    if get_attempt.is_ok() {
        raw_webpage = get_attempt.unwrap();
    } else {
        //the webpage keeps giving errors
        //also not including the time because if this error was not fixable by changing time then replacing the old time is a waste of time
        return Err("Failed Reaching the Initial Profile Page".to_owned());
    }

    let document = scraper::Html::parse_document(&raw_webpage);
    let steam_id: String;
    if type_id == "profiles" {
        steam_id = id.to_owned();
    } else {
        let steam_id_attempt = get_steam_id_from_profile(&document);
        if steam_id_attempt.is_ok() {
            steam_id = steam_id_attempt.unwrap();
        } else {
            return Err(steam_id_attempt.unwrap_err());
        }
    }
    let scraper_selector = scraper::Selector::parse("div.profile_item_links").unwrap();
    let page_items = document.select(&scraper_selector);
    let mut items: Vec<String> = Vec::new();
    // items: ["games", "inventory", "screenshots", "recommended"] by the end of this loop
    // this is now updated to search for numbers
    for page_item in page_items {
        let selector = scraper::Selector::parse("div.profile_count_link").unwrap();
        //redo time
        let items_select = page_item.select(&selector);
        for item in items_select {
            let selector1 = scraper::Selector::parse("span.count_link_label").unwrap();
            let title_list_select: Vec<String> =
                item.select(&selector1).map(|count| count.html()).collect();
            let cleaned_title_list: String = title_list_select
                .concat()
                .replace("\n", "")
                .replace("\t", "");
            let spl1: Vec<&str> = cleaned_title_list.split(">").collect();
            let spl2: Vec<&str> = spl1[1].split("<").collect();
            let title = spl2[0];
            if title == "Games" || title == "Reviews" {
                let selector2 = scraper::Selector::parse("span.profile_count_link_total").unwrap();
                let count_list_select: Vec<String> =
                    item.select(&selector2).map(|count| count.html()).collect();
                let cleaned_count_list: String = count_list_select
                    .concat()
                    .replace("\n", "")
                    .replace("\t", "");
                let spl1: Vec<&str> = cleaned_count_list.split(">").collect();
                let spl2: Vec<&str> = spl1[1].split("<").collect();
                //gotta remove the comma from the thousands(was skipping profiles with over 1000)
                let count_string = spl2[0].replace(",", "");
                let count = count_string.parse::<u32>().unwrap_or_default();
                if title == "Reviews" {
                    if count > 1 {
                        items.push("recommended".to_owned());
                    }
                } else {
                    if count > 0 {
                        items.push("games".to_owned());
                    }
                }
            }
        }
    }
    let games_visable: bool = items.contains(&"games".to_owned());
    let reviews_visable: bool = items.contains(&"recommended".to_owned());
    //adding extra rules to stop wasting time trying to grab empty reviews/ only 1 review(cant see one review)
    Ok((
        Visability {
            games: games_visable,
            reviews: reviews_visable,
        },
        steam_id,
    ))
}
//1. grab games
async fn get_game_list(steam_id: &str, steam_api: &str) -> Result<Vec<Game>, String> {
    let url = format!(
        "http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json",
        steam_api.to_string(),steam_id
    );
    match reqwest::get(&url).await {
        Ok(resp) => {
            let resp_text: String = resp.text().await.unwrap();
            let data: Result<GameListData, serde_json::Error> = serde_json::from_str(&resp_text);
            if data.is_ok() {
                return Ok(data.unwrap().response.games);
            } else {
                return Err(
                    "Failed to convert the game list from json, likely getting a steam api error"
                        .to_owned(),
                );
            }
        }
        Err(_err) => return Err("Failed to grab the game list from the Steam API".to_owned()),
    };
}

fn get_review_page_count(raw_webpage: &str) -> usize {
    let document = scraper::Html::parse_document(&raw_webpage);
    let mut page_count: usize = 1;
    let scraper_selector = scraper::Selector::parse("div.workshopBrowsePagingControls").unwrap();
    let page_number_blocks = document.select(&scraper_selector);
    if !&page_number_blocks.last().is_none() {
        let page_number_blocks = document.select(&scraper_selector);
        let page_number_block = page_number_blocks.last().unwrap();
        let scraper_selector = scraper::Selector::parse("a.pagelink").unwrap();
        let page_number_links: scraper::element_ref::Select<'_, '_> =
            page_number_block.select(&scraper_selector);
        let page_number_links: Vec<String> = page_number_links
            .into_iter()
            .map(|page_nubmer| page_nubmer.html())
            .collect();
        if page_number_links.len() != 0 {
            let final_page_html = page_number_links.last().unwrap();
            let first_split: Vec<&str> = final_page_html.split("p=").collect();
            let second_split: Vec<&str> = first_split[1].split("\"").collect();
            page_count = second_split[0].parse::<usize>().unwrap();
        }
    }
    //page_count += page_number_blocks_html.len();
    page_count
}

async fn get_review_list(steam_id: &str, client: &reqwest::Client) -> Result<Vec<Review>, ()> {
    let mut review_list: Vec<Review> = Vec::new();
    let url = format!("https://steamcommunity.com/profiles/{}/reviews", steam_id);
    let mut raw_webpage: String;
    if let Ok(get_attempt) = get_raw_page_holding_state(&url, 1, &client).await {
        raw_webpage = get_attempt;
    } else {
        //the webpage keeps giving errors
        //also not including the time because if this error was not fixable by changing time then replacing the old time is a waste of time
        return Err(());
    }

    let page_count = get_review_page_count(&raw_webpage);
    let mut current_page_number = 1;
    while current_page_number < page_count + 1 && page_count < 150 {
        if current_page_number > 1 {
            let url = format!(
                "https://steamcommunity.com/profiles/{}/reviews/?p={}",
                steam_id, current_page_number
            );

            if let Ok(get_attempt) = get_raw_page_holding_state(&url, 1, &client).await {
                raw_webpage = get_attempt;
            } else {
                //the webpage keeps giving errors
                //also not including the time because if this error was not fixable by changing time then replacing the old time is a waste of time
                return Err(());
            }
        }
        review_list.append(&mut scrape_review_page(&raw_webpage));
        current_page_number += 1;
    }
    Ok(review_list)
}
fn scrape_review_page(raw_webpage: &str) -> Vec<Review> {
    let document = scraper::Html::parse_document(raw_webpage);
    let mut reviews_on_page: Vec<Review> = Vec::new();
    let scraper_selector = scraper::Selector::parse("div.review_box").unwrap();
    let review_blocks = document.select(&scraper_selector);

    let mut review_a_list: Vec<(String, String)> = Vec::new();
    for element in review_blocks {
        // for recommendations + appID
        let scraper_selector1 = scraper::Selector::parse("div.title").unwrap();
        let review_a = element.select(&scraper_selector1);
        let current_a: Vec<String> = review_a.into_iter().map(|id| id.html()).collect();
        let current_a = current_a.concat();
        // for hours
        let scraper_selector2 = scraper::Selector::parse("div.hours").unwrap();
        let review_b = element.select(&scraper_selector2);
        let current_b: Vec<String> = review_b.into_iter().map(|id| id.html()).collect();
        let current_b = current_b.concat();
        review_a_list.push((current_a, current_b));
    }
    for review in review_a_list {
        // for recommendations + appID
        //<div class=\"title\"><a href=\"https://steamcommunity.com/id/ameobea/recommended/427520/\">Recommended</a></div>
        let split_one: Vec<&str> = review.0.split("recommended/").collect();
        //427520/\">Recommended</a></div>
        let split_two: Vec<&str> = split_one[1].split("/").collect();
        //427520| ">Recommended</a></div>
        let game_id = split_two[0];
        let game_id_num = game_id.parse::<u32>().unwrap();
        //">Recommended</a></div>
        let split_three: Vec<&str> = split_two[1].split(">").collect();

        let split_four: Vec<&str> = split_three[1].split("<").collect();
        let is_recommended: bool = match split_four[0] {
            "Recommended" => true,
            "Not Recommended" => false,
            _ => false,
        };
        //for hours
        let hour_one: Vec<&str> = review.1.split(">").collect();
        let mut cleaned_hour = hour_one[1].replace("/t", "");
        cleaned_hour = cleaned_hour.replace(",", "");
        let hour_two: Vec<&str> = cleaned_hour.split("hrs").collect();
        let hour_string: &str = hour_two[0].trim();
        let minutes: u32 = match hour_string {
            "" => 0,
            _ => (hour_string.parse::<f32>().unwrap_or_default() * 60 as f32) as u32,
        };
        let current_review: Review = Review {
            game_id: game_id_num,
            is_recommended,
            time_played: minutes,
        };
        reviews_on_page.push(current_review);
    }
    reviews_on_page
}
fn combine_games_and_reviews(games: Vec<Game>, reviews: Vec<Review>) -> Vec<Game> {
    if reviews.len() == 0 {
        return games;
    }
    let mut new_games = games.clone();
    for review in reviews {
        let recommendation_i8 = match review.is_recommended {
            true => 1,
            false => -1,
        };
        let position_in_games = games
            .clone()
            .into_iter()
            .position(|game| game.appid == review.game_id)
            .unwrap_or(usize::MAX);
        if position_in_games == usize::MAX {
            new_games.push(Game {
                appid: review.game_id,
                playtime_2weeks: 0,
                playtime_forever: review.time_played,
                is_recommended: recommendation_i8,
            })
        } else {
            new_games[position_in_games].is_recommended = recommendation_i8;
        }
    }
    new_games
}
fn games_and_reviews_into_scorelist(
    input_combination: Vec<Game>,
    classifications: &Vec<GameInfo>,
) -> (Vec<f32>, Vec<usize>) {
    let mut list_of_existing_indexs: Vec<usize> = Vec::new();
    let current_score_list: Vec<GameInfo> = classifications.clone();
    let mut output: Vec<f32> = Vec::new();
    let current_score_list_len = current_score_list.len();
    for x in 0..current_score_list_len {
        let game_found = input_combination
            .clone()
            .into_iter()
            .find(|input| input.appid == current_score_list[x].steam_appid);
        if game_found.is_some() {
            list_of_existing_indexs.push(x);
            output.push(data_to_score(game_found.unwrap()));
        } else {
            output.push(0.)
        }
    }
    return (output, list_of_existing_indexs);
    //check if each one of the classification games are in the list or give it a zero
}
fn data_to_score(game: Game) -> f32 {
    //playtime_2weeks: f32, playtime_forever: f32, is_recommended: i8
    let playtime_forever = game.playtime_forever as f32;
    let mut new_score: f32 = 0.;
    new_score += (3 * game.is_recommended) as f32;
    // playtime_forever -> 0.0, +6.0
    //60(1hour), 600(10hours), 6000(100hours),30000(500hours), 600000(1000hours)
    // going back to try and avoid pushign a zero score to add some more variance
    if playtime_forever < 60. {
        new_score += playtime_forever / 60.;
    } else if playtime_forever < 600. {
        new_score += 1. + (playtime_forever / 600.);
    } else if playtime_forever < 6000. {
        new_score += 2. + (playtime_forever / 6000.);
    } else if playtime_forever < 30000. {
        new_score += 4. + (playtime_forever / 30000.);
    } else if playtime_forever < 60000. {
        new_score += 5. + (playtime_forever / 60000.);
    }
    // playtime_2weeks -> 0.0, +3.0
    // a point for every 2 hours that stops at 6 hours
    let mut two_week_bonus: f32 = game.playtime_2weeks as f32 / 120.;
    if two_week_bonus > 3. {
        two_week_bonus = 3.;
    }
    //totals to worst:-3 +0 + 0 = -3 or best 3+6+3 = 12
    new_score += two_week_bonus;
    new_score
}
fn build_request_string_from_array(input: Vec<f32>) -> String {
    let mut output: String = "".to_owned();
    let input_len = input.len();
    for x in 0..(input_len - 1) {
        output += &input[x].to_string();
        output += ",";
    }
    output += &input[input_len - 1].to_string();
    output
}

struct IndividualState {
    client: reqwest::Client,
    classifications: Vec<GameInfo>,
    steam_api: String,
}
#[launch]
fn rocket() -> _ {
    dotenv().ok();
    rocket::build()
        .attach(CORS)
        .manage(IndividualState {
            client: reqwest::Client::new(),
            classifications: {
                println!("Checking for classifications");
                tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(async { get_classification_game_list().await })
            },
            steam_api: var("STEAM_API_KEY").expect("need to have .env file with steam api in it"),
        })
        .configure(rocket::Config::figment().merge(("port", 5001)))
        .mount("/", routes![convert_link])
}
