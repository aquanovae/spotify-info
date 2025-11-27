use anyhow::Result;
use env_logger::{
    fmt::Target,
    Builder,
};
use log::LevelFilter;
use rspotify::{
    clients::OAuthClient,
    model::{
        enums::types::AdditionalType,
        PlayableItem,
    },
    AuthCodePkceSpotify, Config, Credentials, OAuth
};
use std::path::PathBuf;


type Spotify = AuthCodePkceSpotify;


const CLIENT_ID: &str = "c3a54b1a058a4ea3a225a27dc8d06b44";
const REDIRECT_URI: &str = "http://127.0.0.1:8080";
const CACHE_PATH: &str = "/tmp/spotify-info";
const TOKEN_CACHE_PATH: &str = "/tmp/spotify-info/token.json";


fn main() {
    Builder::new()
        .target(Target::Stderr)
        .filter_level(LevelFilter::Error)
        .init();

    match std::fs::create_dir_all(CACHE_PATH) {
        Ok(_) => (),
        Err(why) => {
            log::error!("{}", why);
            return;
        }
    }

    let spotify = match authenticate() {
        Ok(spotify) => spotify,
        Err(why) => {
            log::error!("{}", why);
            return;
        }
    };

    match get_playing_info(&spotify) {
        Some(info) => {
            println!("{}", info);
        },
        None => {
            return;
        }
    };
}


fn authenticate() -> Result<Spotify> {
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
    let mut spotify = Spotify::with_config(creds, oauth, config);
    let authorize_url = spotify.get_authorize_url(Some(128))?;
    spotify.prompt_for_token(&authorize_url)?;
    Ok(spotify)
}


fn get_playing_info(spotify: &Spotify) -> Option<String> {
    let context = match spotify.current_playing(None, None::<Vec<&AdditionalType>>) {
        Ok(context) => context,
        Err(why) => {
            log::error!("{}", why);
            return None;
        }
    };
    let context = match context {
        Some(context) => context,
        None => {
            return None;
        }
    };
    let item = match context.item {
        Some(item) => item,
        None => {
            return None;
        }
    };
    let track = match item {
        PlayableItem::Track(track) => track,
        _ => {
            return None;
        }
    };
    let artists = track.artists
        .iter()
        .map(|artist| artist.name.clone())
        .collect::<Vec<_>>();
    let track_info = format!("{} - {}", artists.join(", "), track.name);
    Some(track_info)
}
