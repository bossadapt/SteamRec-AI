use dotenv::dotenv;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Header, Status};
use rocket::response::{content, status};
use rocket::serde::json::{serde_json, Json};
use rocket::tokio::time::sleep;
use rocket::{Request, Response};
use scraper::{node::Element, ElementRef, Html};
use serde::{Deserialize, Serialize};
use std::env::var;
use std::string;
use std::time::Duration;
#[macro_use]
extern crate rocket;
extern crate reqwest;
#[derive(Debug)]
struct Visability {
    games: bool,
    reviews: bool,
}
// TYPES FOR API AND SCRAPER////////////
#[derive(Debug, Deserialize)]
struct Data {
    response: GameRequest,
}
#[derive(Debug, Deserialize)]
struct GameRequest {
    game_count: serde_json::Number,
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
////////////////////////////////////////
#[derive(Serialize)]
struct ResponseSucess {
    test: String,
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

#[get("/convert/<link>")]
async fn convert_link(link: &str) -> Json<ResponseSucess> {
    println!("link recieved: {}", link);
    let test_set: Vec<f32> = [0.0; 9800].to_vec();
    let req = format!(
        "http://127.0.0.1:5002/predict/{}",
        build_request_string_from_array(test_set)
    );
    let python_predict = reqwest::get(req).await.unwrap().text().await.unwrap();
    println!("request finished: {}", python_predict);
    Json(ResponseSucess {
        test: python_predict,
    })
}

async fn get_raw_page(url: String) -> Result<String, String> {
    sleep(Duration::from_secs(1));
    match reqwest::get(&url).await {
        Ok(resp) => return Ok(resp.text().await.unwrap()),
        Err(_err) => return Err(format!("Failed to grab raw webpage at: {}", url)),
    }
}
//proper link will be checked if

//1. visit profile and ensure it exists
//2. ensure games and reviews visability
async fn get_visibility(steam_id: &str) -> Result<Visability, String> {
    //check that freinds, games and reviews are visable via their home profile
    let url = format!("https://steamcommunity.com/profiles/{}/", steam_id);
    let get_attempt = get_raw_page(url).await;
    let raw_webpage: String;
    if get_attempt.is_ok() {
        raw_webpage = get_attempt.unwrap();
    } else {
        //the webpage keeps giving errors
        //also not including the time because if this error was not fixable by changing time then replacing the old time is a waste of time
        return Err("Failed Reaching the Initial Profile Page".to_owned());
    }
    let document = scraper::Html::parse_document(&raw_webpage);
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
                    if count > 1 && count < 1500 {
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
    Ok(Visability {
        games: games_visable,
        reviews: reviews_visable,
    })
}
//1. grab games
async fn get_game_list(steam_id: &str, steam_api: &String) -> Result<Vec<Game>, String> {
    let url = format!(
        "http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json",
        steam_api,steam_id
    );
    match reqwest::get(&url).await {
        Ok(resp) => {
            let resp_text: String = resp.text().await.unwrap();
            let data: Result<Data, serde_json::Error> = serde_json::from_str(&resp_text);
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

fn get_review_page_count(document: Html) -> usize {
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

async fn get_review_list(steam_id: &str) -> Result<Vec<Review>, String> {
    let mut review_list: Vec<Review> = Vec::new();
    let url = format!("https://steamcommunity.com/profiles/{}/reviews", steam_id);
    let get_attempt = get_raw_page(url).await;
    let raw_webpage: String;
    if get_attempt.is_ok() {
        raw_webpage = get_attempt.unwrap();
    } else {
        //the webpage keeps giving errors
        //also not including the time because if this error was not fixable by changing time then replacing the old time is a waste of time
        return Err(get_attempt.unwrap_err());
    }
    let mut document = scraper::Html::parse_document(&raw_webpage);
    let page_count = get_review_page_count(document.clone());
    let mut current_page_number = 1;
    while current_page_number < page_count + 1 && page_count < 150 {
        if current_page_number > 1 {
            let url = format!(
                "https://steamcommunity.com/profiles/{}/reviews/?p={}",
                steam_id, current_page_number
            );
            let get_attempt = get_raw_page(url).await;
            let raw_webpage: String;
            if get_attempt.is_ok() {
                raw_webpage = get_attempt.unwrap();
            } else {
                return Err(get_attempt.unwrap_err());
            }
            document = scraper::Html::parse_document(&raw_webpage);
        }
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
            review_list.push(current_review);
        }
        current_page_number += 1;
    }
    println!("      added page(s) of reviews");
    Ok(review_list)
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
fn reviews_and_games_into_scorelist(Vec<Game>) -> Vec<f32> {
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

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    let steam_api: String =
        var("STEAM_API_KEY").expect("need to have .env file with steam api in it");
    rocket::build()
        .attach(CORS)
        .configure(rocket::Config::figment().merge(("port", 5001)))
        .mount("/", routes![convert_link])
}
