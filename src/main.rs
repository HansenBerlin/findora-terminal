use rppal::gpio::Gpio;
use std::time::Duration;
use std::env;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::error::Error;
use serde::Serialize;

#[allow(non_snake_case)]
#[derive(Serialize)]
pub struct NewGameRequest {
    pub redTeamName: String,
    pub blueTeamName: String,
    pub gameLength: u64,
}

const GPIO_RED: u8 = 5;    // Pin 29 — stop game
const GPIO_BLUE: u8 = 6;   // Pin 31 — start game

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let endpoint = env::var("BUTTON_API_ENDPOINT")
        .map_err(|_| "BUTTON_API_ENDPOINT not set")?;
    let api_key = env::var("BUTTON_API_KEY")
        .map_err(|_| "BUTTON_API_KEY not set")?;

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {api_key}"))?);

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    let base_url = format!("{endpoint}/api/");
    let health_url = format!("{base_url}health");
    let new_game_url = format!("{base_url}game/new");
    let stop_game_url = format!("{base_url}game/stop");

    client
        .get(&health_url)
        .send()
        .await?
        .error_for_status()
        .map_err(|_| "Couldn't reach the server!")?;

    let gpio = Gpio::new()?;
    let red_button = gpio.get(GPIO_RED)?.into_input_pullup();
    let blue_button = gpio.get(GPIO_BLUE)?.into_input_pullup();

    println!("Button listener started.");
    println!("  BLUE (GPIO{GPIO_BLUE}, Pin 31) = Start game");
    println!("  RED  (GPIO{GPIO_RED}, Pin 29)  = Stop game");

    let mut game_active = false;

    loop {
        if blue_button.is_low() && !game_active {
            println!("BLUE pressed — starting new game...");

            let res = client
                .post(&new_game_url)
                .json(&NewGameRequest {
                    redTeamName: "red".to_string(),
                    blueTeamName: "blue".to_string(),
                    gameLength: 300,
                })
                .send()
                .await?;

            match res.error_for_status() {
                Ok(_) => {
                    game_active = true;
                    println!("Game started.");
                }
                Err(e) => println!("Failed to start game: {e}"),
            }

            while blue_button.is_low() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        if red_button.is_low() && game_active {
            println!("RED pressed — stopping game...");

            let res = client
                .post(&stop_game_url)
                .send()
                .await?;

            match res.error_for_status() {
                Ok(_) => {
                    game_active = false;
                    println!("Game stopped.");
                }
                Err(e) => println!("Failed to stop game: {e}"),
            }

            while red_button.is_low() {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}