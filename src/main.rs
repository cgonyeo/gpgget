extern crate hyper;
extern crate rand;
extern crate docopt;
extern crate rustc_serialize;

use std::io;
use std::fs::File;
use hyper::Client;
use std::env;
use std::path;
use rand::Rng;
use docopt::Docopt;
use std::process::*;
use std::fs;

static USAGE: &'static str = "
Usage: gpgget [options] <url>
       gpgget (-h | --help)

Options:
    --gpg <binary>  command to use instead of 'gpg'
    -o <file>, --output=<file>  write output to <file> instead of stdout
    -h, --help  display help page
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_url: String,
    flag_output: Option<String>,
    flag_gpg: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    let url = args.arg_url;

    let client = Client::new();

    let gpg = match args.flag_gpg {
        Some(g) => g,
        None => String::from("gpg"),
    };

    let tmp_file = http_get_to_tmp_file(&client, &url).unwrap();
    let tmp_sig_file = http_get_to_tmp_file(&client, &(url+".asc")).unwrap();

    let output = Command::new(gpg)
                         .arg("--verify")
                         .arg(&tmp_sig_file)
                         .arg(&tmp_file)
                         .output()
                         .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

    if !output.status.success() {
        fs::remove_file(tmp_file).unwrap();
        fs::remove_file(tmp_sig_file).unwrap();
        println!("{}", String::from_utf8_lossy(&output.stderr));
        exit(output.status.code().unwrap());
    }
}

fn get_tmp_file() -> path::PathBuf {
    let mut rng = rand::thread_rng();
    let mut dir = env::temp_dir();
    dir.push(rng.gen::<u32>().to_string());
    return dir;
}

fn http_get_to_tmp_file(client : &Client, url : &String) -> Option<path::PathBuf> {
    println!("{}", url);
    let mut res = client.get(url).send().unwrap();
    assert_eq!(res.status, hyper::Ok);

    let tmp_file = get_tmp_file();
    let mut file = File::create(&tmp_file).unwrap();

    io::copy(&mut res, &mut file).unwrap();

    return Some(tmp_file);
}
