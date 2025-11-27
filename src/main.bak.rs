use anyhow::Result;
use rand::seq::{ IndexedRandom, SliceRandom };
use rspotify::{
    clients::{ BaseClient, OAuthClient },
    model::{
        idtypes::{ PlayableId, PlaylistId, TrackId },
        track::FullTrack,
        PlayableItem,
    },
    AuthCodePkceSpotify, Config, Credentials, OAuth
};
use std::{
    collections::HashMap,
    io::ErrorKind,
    path::PathBuf,
};


type Spotify = AuthCodePkceSpotify;
type TrackList = Vec<TrackId<'static>>;
type Weights = HashMap<TrackId<'static>, usize>;


const CLIENT_ID: &str = "afc50da0ba7b4042bf56e1a2c72a784e";
const REDIRECT_URI: &str = "http://127.0.0.1:8080";
const API_REQUEST_MAX: usize = 100;

const CACHE_DIR: &str = "/tmp/spotify-daily";
const CACHE_TOKEN: &str = "/tmp/spotify-daily/token.json";
const CACHE_WEIGHTS: &str = "/tmp/spotify-daily/weights.json";

const PLAYLIST_COLLECTION_ID: &str = "2RLVL6f3CCm7LD57wbwx2j";
const PLAYLIST_CURRENT_ID: &str = "77JTZoDLsmXm1ODTdVc1oz";
const PLAYLIST_DAILY_ID: &str = "42O1aSlfF0vlmLuBkPlcDO";
const PLAYLIST_SIZE: usize = 120;


fn main() -> Result<()> {
    let mut generator = Generator::init()?;
    generator.make_selection()?;
    generator.update_weights()?;
    generator.update_playlist()?;
    Ok(())
}


struct Generator {
    spotify: Spotify,
    weights: Weights,
    collection: TrackList,
    selection: TrackList,
}


impl Generator {

    fn init() -> Result<Generator> {
        let mut spotify = Generator::authenticate()?;
        let weights = Generator::open_weights()?;
        let collection = Generator::fetch_playlist(
            &mut spotify, PLAYLIST_COLLECTION_ID
        )?;
        let selection = Generator::fetch_playlist(
            &mut spotify, PLAYLIST_CURRENT_ID
        )?;
        Ok(Generator{
            spotify,
            weights,
            collection,
            selection,
        })
    }

    fn authenticate() -> Result<Spotify> {
        let credentials = Credentials::new_pkce(CLIENT_ID);
        let oauth = OAuth{
            redirect_uri: String::from(REDIRECT_URI),
            scopes: rspotify::scopes!(
                "playlist-read-private",
                "playlist-modify-private"
            ),
            ..Default::default()
        };
        std::fs::create_dir_all(&CACHE_DIR)?;
        let cache_path = PathBuf::from(CACHE_TOKEN);
        let config = Config{
            cache_path,
            token_cached: true,
            token_refreshing: true,
            ..Default::default()
        };
        let mut spotify = Spotify::with_config(credentials, oauth, config);
        let authorize_url = spotify.get_authorize_url(Some(128))?;
        spotify.prompt_for_token(&authorize_url)?;
        Ok(spotify)
    }

    fn open_weights() -> Result<Weights> {
        let history = match std::fs::read_to_string(CACHE_WEIGHTS) {
            Ok(file_content) => serde_json::from_str(&file_content)?,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => Weights::new(),
                _ => return Err(error.into()),
            },
        };
        Ok(history)
    }

    fn fetch_playlist(spotify: &mut Spotify, id: &str) -> Result<TrackList> {
        let playlist_id = PlaylistId::from_id(id)?;
        let track_list = spotify
            .playlist_items(playlist_id, None, None)
            .flatten()
            .filter_map(|item| match item.track {
                Some(PlayableItem::Track(FullTrack{ id, .. })) => id,
                _ => None,
            })
            .collect::<Vec<_>>();
        Ok(track_list)
    }

    fn make_selection(&mut self) -> Result<()> {
        let sample_size = PLAYLIST_SIZE - self.selection.len();
        let mut rng = rand::rng();
        let mut selection = self.collection
            .choose_multiple_weighted(&mut rng, sample_size, |track| {
                match self.weights.get(track) {
                    Some(weight) => *weight as f64,
                    None => 1.0,
                }
            })?
            .cloned()
            .collect::<TrackList>();
        self.selection.append(&mut selection);
        for _ in 0..7 {
            self.selection.shuffle(&mut rng);
        }
        Ok(())
    }

    fn update_weights(&mut self) -> Result<()> {
        for (_, weight) in self.weights.iter_mut() {
            *weight += 1;
        }
        for track in self.collection.iter() {
            if !self.weights.contains_key(track) {
                self.weights.insert(track.clone(), 1);
            }
        }
        for track in self.selection.iter() {
            if let Some(weight) = self.weights.get_mut(track) {
                *weight = 1;
            }
        }
        let file_content = serde_json::to_string(&self.weights)?;
        std::fs::write(CACHE_WEIGHTS, &file_content)?;
        Ok(())
    }

    fn update_playlist(&mut self) -> Result<()> {
        let id = PlaylistId::from_id(PLAYLIST_DAILY_ID)?;
        let old_track_list = {
            Generator::fetch_playlist(&mut self.spotify, PLAYLIST_DAILY_ID)?
                .into_iter()
                .map(|track| PlayableId::from(track))
                .collect::<Vec<_>>()
        };
        for chunk in old_track_list.chunks(API_REQUEST_MAX) {
            let chunk = chunk
                .into_iter()
                .cloned();
            self.spotify.playlist_remove_all_occurrences_of_items(
                id.clone(), chunk, None
            )?;
        }
        let new_track_list = self.selection
            .iter()
            .map(|track| PlayableId::from(track.clone()))
            .collect::<Vec<_>>();
        for chunk in new_track_list.chunks(API_REQUEST_MAX) {
            let chunk = chunk
                .into_iter()
                .cloned();
            self.spotify .playlist_add_items(id.clone(), chunk, None)?;
        }
        Ok(())
    }
}
