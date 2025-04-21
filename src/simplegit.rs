use git2::{Cred, RemoteCallbacks};
use ssh2_config::{SshConfig, ParseRule};
use std::path::{Path};

pub fn clone_repo(url: String){
    let config = SshConfig::parse_default_file(ParseRule::STRICT).expect("Failed to parse configuration");

     let s1: Vec<_> = url.split('@').collect();
    if s1.len() <= 1 { panic!("URL should be of the form of: git@github.com:<USERNAME>/REPOSITORY --> s1 {s1:?}");}

    let repo_url = s1[1];
    
    let s2: Vec<_> = repo_url.split(':').collect();
    if s2.len() <= 1 { panic!("URL should be of the form of: git@github.com:<USERNAME>/REPOSITORY --> s2 {s2:?}");}
    let host_url = s2[0];

    println!("Repository URL: {repo_url}");
    println!("Host URL: {host_url}");

    let host_params = config.query(host_url);
    
    let _user = host_params.user;
    let priv_key = host_params.identity_file;
    let _hostname = host_params.host_name;

    /* println!("User {:?} | Identity File: {:?} | Hostname {:?}", user, priv_key.clone().unwrap(), hostname); */

    let cred = match Cred::ssh_key_from_agent("personal") {
        Ok(c) => c,
        Err(err) => panic!("{}", err),
    };

    println!("Has username: {}", cred.has_username());

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            priv_key.clone().unwrap()[0].as_path(),
            None,
        )
        // Cred::ssh_key_from_agent("salorak")
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);

    let final_url = String::from("git@github.com:") + s2[1];
    println!("URL --> {}", final_url);
    let repo = builder.clone(
        final_url.as_str(),
        Path::new("/home/hector/test/dottest")
    );

    match repo {
        Ok(_repo) => println!("Lets go!!"),
        Err(err) => eprintln!("ERROR! {}", err),
    }
        
}
