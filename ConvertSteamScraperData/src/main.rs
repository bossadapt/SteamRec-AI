use rusqlite::Connection;
use serde::Deserialize;
#[derive(Deserialize)]
struct Game {
    appid: u32,
    playtime_2weeks: u16,
    playtime_forever: u32,
    is_recommended: i8,
}
struct Account {
    steam_id: u64,
    games_used: bool,
    friends_available: bool,
    friends: Vec<String>,
    games: Vec<Game>,
}
#[derive(Deserialize)]
struct OldDataOutput {
    steam_id: u64,
    games: String,
}
#[derive(Clone)]
struct OldDataOutputConverted {
    row_id: String,
    steam_id: String,
    app_id: String,
    playtime_2weeks: f32,
    playtime_forever: f32,
    is_recommended: i8,
}
#[derive(Clone)]
struct FinalizeDataOutput {
    row_id: String,
    steam_id: String,
    app_id: usize,
    score: f32,
}
#[derive(Clone)]
struct GameCount {
    app_id: String,
    count: u32,
}
// For filtering from initial list
const MINIMUM_GAME_OCCURANCE: u32 = 1000;
const MINIMUM_TIME_PLAYED: u32 = 5;
const SCRAPER_SQLITE_PATH: &str = "D:\\data\\users";
const REFOREMED_SQLITE_PATH: &str = "D:\\data\\reformedUserData";
// weights for each part (based on percentage of users time totals)
fn main() {
    // the initial generate needs to run before the second at least once
    generate_starter_list();
}
fn data_to_score(playtime_2weeks: f32, playtime_forever: f32, is_recommended: i8) -> f32 {
    let mut new_score: f32 = 0.;
    // is_recommended -> +3 , -3 , 0
    new_score += (3 * is_recommended) as f32;
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
    let mut two_week_bonus: f32 = playtime_2weeks / 120.;
    if two_week_bonus > 3. {
        two_week_bonus = 3.;
    }
    //totals to worst:-3 +0 + 0 = -3 or best 3+6+3 = 12
    new_score += two_week_bonus;
    new_score
}
fn generate_starter_list() {
    let old_data_con = Connection::open(SCRAPER_SQLITE_PATH).expect("cant find read file");
    let new_data_conn = Connection::open(REFOREMED_SQLITE_PATH).expect("cant find write file");

    //BUILDING THE FILTERED AND REFACTORED LIST
    println!("Pulling Data From the database");
    let mut stmt = old_data_con
        .prepare("SELECT steamID,games FROM accounts WHERE games_used = 1")
        .unwrap();
    let old_data_iter = stmt
        .query_map([], |row| {
            Ok(OldDataOutput {
                steam_id: row.get(0).unwrap(),
                games: row.get(1).unwrap(),
            })
        })
        .unwrap();
    println!("-Finished pulling Data From the database");
    println!("creating new sql tables and templates");
    new_data_conn
        .execute("DROP TABLE IF EXISTS gameInteractions", [])
        .unwrap();
    new_data_conn.execute("CREATE TABLE gameInteractions( row_id TEXT PRIMARY KEY, steam_id INTEGER, app_id INTEGER, score REAL)",[]).expect("main table creation failed");
    let mut smnt = new_data_conn
        .prepare("INSERT INTO gameInteractions VALUES (?1,?2,?3,?4)")
        .unwrap();
    println!("-Finished creating new sql tables and templates");
    println!("Reformatting scraper data away from json");
    //converted to new format
    let mut new_data_list: Vec<OldDataOutputConverted> = Vec::new();
    for data in old_data_iter {
        let current = data.unwrap();
        let game_list: Vec<Game> = serde_json::from_str(current.games.as_str()).unwrap();
        for game in game_list {
            if game.playtime_forever > 0 {
                new_data_list.push(OldDataOutputConverted {
                    row_id: format!(
                        "{}.{}",
                        current.steam_id.to_string(),
                        game.appid.to_string()
                    ),
                    steam_id: current.steam_id.to_string(),
                    app_id: game.appid.to_string(),
                    playtime_2weeks: game.playtime_2weeks as f32,
                    playtime_forever: game.playtime_forever as f32,
                    is_recommended: game.is_recommended,
                })
            }
        }
    }
    println!("-Finished Reformatting scraper data");
    //look through new format for game counts to create a smaller classification list < 10000
    println!("Finding Classification list");
    let mut game_counts: Vec<GameCount> = Vec::new();
    for line in &mut new_data_list {
        if line.playtime_forever < MINIMUM_TIME_PLAYED as f32 {
            continue;
        }
        let mut index_found: i32 = -1;
        for (index, game) in game_counts.iter().enumerate() {
            if game.app_id == line.app_id {
                index_found = index as i32;
                break;
            }
        }
        if index_found != -1 {
            game_counts[index_found as usize].count += 1;
        } else {
            game_counts.push(GameCount {
                app_id: line.app_id.to_owned(),
                count: 1,
            })
        }
    }

    //classification table built
    let mut classification_list: Vec<(String, u32)> = Vec::new();
    for game in game_counts {
        if game.count > MINIMUM_GAME_OCCURANCE {
            classification_list.push((game.app_id, game.count));
        }
    }
    println!("-Finished Finding Classification list");
    println!("pushing classification to sql");
    new_data_conn.execute("BEGIN TRANSACTION", []).unwrap();
    new_data_conn
        .execute("DROP TABLE IF EXISTS classifications", [])
        .unwrap();
    new_data_conn
        .execute(
            "CREATE TABLE classifications( gameID TEXT PRIMARY KEY, count INTEGER)",
            [],
        )
        .unwrap();
    let mut insert_statement_for_classificaiton = new_data_conn
        .prepare("INSERT INTO classifications Values(?1,?2)")
        .unwrap();
    for item in classification_list.clone() {
        insert_statement_for_classificaiton
            .execute([item.0, item.1.to_string()])
            .unwrap();
    }
    new_data_conn.execute("COMMIT", []).unwrap();
    println!("-finished pushing classification to sql");
    //purge games that do not fit the rules(classification table)
    //push the rest(replacing the game id with an index)
    //replace 3 entries with a scoring system
    let mut counter_output = 0;
    println!("Filter 1 started(games):{}", new_data_list.len());
    let mut filtered_new_data: Vec<FinalizeDataOutput> = Vec::new();
    for data in new_data_list {
        // get rid of games that have no time played
        for i in 0..classification_list.len() {
            if classification_list[i].0 == data.app_id {
                let score = data_to_score(
                    data.playtime_2weeks as f32,
                    data.playtime_forever as f32,
                    data.is_recommended,
                );
                counter_output += 1;
                filtered_new_data.push(FinalizeDataOutput {
                    row_id: data.row_id,
                    steam_id: data.steam_id,
                    app_id: i,
                    score: score,
                });
                break;
            }
        }
    }
    println!("-Finished Filter 1 started(games):{}", counter_output);
    //purge accounts with less than 2 games and convert steam IDs to a number index
    let mut final_filtered_new_data: Vec<FinalizeDataOutput> = Vec::new();
    let data_length = counter_output;
    let mut prev_steam_id = "".to_owned();

    println!("Filter 2 started(steam accounts):{}", data_length);
    //making sure the first +1 makes the index start at 0
    let mut steam_id_index = -1;
    for (i, line) in filtered_new_data.clone().into_iter().enumerate() {
        if prev_steam_id == line.steam_id {
            //identical steam id was found prior
            prev_steam_id = line.steam_id.to_owned();
            final_filtered_new_data.push(FinalizeDataOutput {
                row_id: line.row_id,
                steam_id: steam_id_index.to_string(),
                app_id: line.app_id,
                score: line.score,
            });
        } else if i + 1 < data_length && filtered_new_data[i + 1].steam_id == line.steam_id {
            //identical steam id was found
            steam_id_index += 1;
            prev_steam_id = line.steam_id.to_owned();
            final_filtered_new_data.push(FinalizeDataOutput {
                row_id: line.row_id,
                steam_id: steam_id_index.to_string(),
                app_id: line.app_id,
                score: line.score,
            });
        }
    }
    println!(
        "-Finished Filter 2 (steam accounts):{}",
        final_filtered_new_data.len()
    );
    println!("Pushing to sql");
    new_data_conn.execute("BEGIN TRANSACTION", []).unwrap();
    let mut final_counter = 0;
    for data in final_filtered_new_data {
        final_counter += 1;
        smnt.execute([
            data.row_id,
            data.steam_id,
            data.app_id.to_string(),
            data.score.to_string(),
        ])
        .unwrap();
    }
    println!("-Finished pushing to sql");

    new_data_conn.execute("COMMIT", []).unwrap();
    println!("length of full data list: {}", final_counter);
    //Refactor the previous list to only hold possible classification outcomes
}
