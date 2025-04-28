use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use axum::{response::Html, Form};
use git2::{Cred, PushOptions, RemoteCallbacks, Repository, Signature};
use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Deserialize)]
pub struct ConcertForm {
    title: String,
    venue: String,
    date: String,
    support: String,
    festival: String,
    token: String,
}

#[derive(Deserialize, Serialize)]
struct Concert {
    title: String,
    date: String,
    venue: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    support: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    festival: String,
}

fn add_concert(form: &ConcertForm) {
    let mut file = File::open("./cv/_data/concerts.yaml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let mut concerts: Vec<Concert> = serde_yaml::from_str(&contents).unwrap();

    // Need to parse the date to a format that can be used in the yaml file
    let parsed_date = form.date.parse::<chrono::NaiveDate>().unwrap();
    let date = parsed_date.format("%d.%m.%Y").to_string();

    concerts.insert(
        0,
        Concert {
            title: form.title.clone(),
            date,
            venue: form.venue.clone(),
            support: form.support.clone(),
            festival: form.festival.clone(),
        },
    );
    let updated_concerts = serde_yaml::to_string(&concerts).unwrap();
    let mut file = File::create("./cv/_data/concerts.yaml").unwrap();
    file.write_all(updated_concerts.as_bytes()).unwrap();
}

pub async fn add(Form(form): Form<ConcertForm>) -> Html<String> {
    // Clone the repo to a temporary directory
    let _ = std::fs::remove_dir_all("./cv");
    let url = "https://github.com/Kyrremann/concert-vitae.git";
    let repo = match Repository::clone(&url, "./cv") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };

    add_concert(&form);

    // Adding the change to the index
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("_data/concerts.yaml")).unwrap();
    index.write().unwrap();

    // Creating a commit
    let oid = index.write_tree().unwrap();
    let author =
        Signature::now("Concert Bot", "cv-scaleway[bot]@users.noreply.github.com").unwrap();
    let tree = repo.find_tree(oid).unwrap();
    let commit_message = format!("concert: {} at {} on {}", form.title, form.venue, form.date);
    let _ = repo
        .commit(
            Some("HEAD"),
            &author,
            &author,
            commit_message.as_str(),
            &tree,
            &[&repo.head().unwrap().peel_to_commit().unwrap()],
        )
        .unwrap();

    // // Pushing the changes to the remote repository
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| Cred::userpass_plaintext(&form.token, ""));

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    let mut remote = repo.find_remote("origin").unwrap();
    remote
        .push(
            &[&format!("refs/heads/main:refs/heads/main")],
            Some(&mut push_options),
        )
        .unwrap();

    Html(format!(
        "Title: {}, Description: {}, Date: {}, Images: {}",
        form.title, form.venue, form.date, form.support
    ))
}
