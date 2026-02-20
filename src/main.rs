use anyhow::Result;
use clap::Parser;
use env_logger::Builder;
use env_logger::fmt::Target;
use log::LevelFilter;
use rspotify::{ AuthCodePkceSpotify, Config, Credentials, OAuth, Token };
use rspotify::clients::OAuthClient;
use rspotify::model::PlayableItem;
use rspotify::model::enums::types::AdditionalType;
use std::path::PathBuf;


type Spotify = AuthCodePkceSpotify;


const CLIENT_ID: &str = "c3a54b1a058a4ea3a225a27dc8d06b44";
const REDIRECT_URI: &str = "http://127.0.0.1:8080";
const CACHE_PATH: &str = "/tmp/spotify-info";
const TOKEN_CACHE_PATH: &str = "/tmp/spotify-info/token.json";


#[derive(Parser)]
struct Cli {
    #[arg(short, default_value_t = false)]
    authenticate: bool,

    #[arg(short, default_value_t = false)]
    verbose: bool,
}


fn main() {
    let cli = Cli::parse();
    
    let log_target = match cli.verbose {
        true => Target::Stdout,
        false => Target::Stderr,
    };

    Builder::new()
        .target(log_target)
        .filter_level(LevelFilter::Error)
        .init();

    match std::fs::create_dir_all(CACHE_PATH) {
        Ok(_) => {}
        Err(why) => {
            log::error!("{}", why);
            return
        }
    }


    let spotify = match get_api(cli.authenticate) {
        Ok(spotify) => spotify,
        Err(why) => {
            log::error!("{}", why);
            return
        }
    };


    match get_playing_info(&spotify) {
        Some(info) => println!("{}", info),
        None => {}
    }
}


fn get_api(auth: bool) -> Result<Spotify> {
    match auth {
        true => authenticate(),
        false => from_cache(),
    }
}


fn authenticate() -> Result<Spotify> {
    let (creds, oauth, config) = pkce_config();
    let mut spotify = Spotify::with_config(creds, oauth, config);
    let authorize_url = spotify.get_authorize_url(Some(128))?;
    spotify.prompt_for_token(&authorize_url)?;
    Ok(spotify)
}


fn from_cache() -> Result<Spotify> {
    let token = Token::from_cache(TOKEN_CACHE_PATH)?;
    let (creds, oauth, config) = pkce_config();
    let spotify = Spotify::from_token_with_config(token, creds, oauth, config);
    Ok(spotify)
}


fn pkce_config() -> (Credentials, OAuth, Config) {
    let creds = Credentials::new_pkce(CLIENT_ID);
    let oauth = OAuth{
        redirect_uri: REDIRECT_URI.to_string(),
        scopes: rspotify::scopes! [
            "user-read-playback-state"
        ],
        ..Default::default()
    };
    let config = Config{
        cache_path: PathBuf::from(TOKEN_CACHE_PATH),
        token_cached: true,
        token_refreshing: true,
        ..Default::default()
    };
    (creds, oauth, config)
}


fn get_playing_info(spotify: &Spotify) -> Option<String> {
    let context = match spotify.current_playing(None, None::<Vec<&AdditionalType>>) {
        Ok(Some(context)) => context,
        Ok(None) => return None,
        Err(why) => {
            log::error!("{}", why);
            return None
        }
    };
    let track = match context.item {
        Some(PlayableItem::Track(track)) => track,
        _ => return None
    };
    let artists = track.artists
        .iter()
        .map(|artist| artist.name.as_str())
        .collect::<Vec<_>>();
    let track_info = format!("{} - {}", artists.join(", "), track.name);
    Some(track_info)
}
