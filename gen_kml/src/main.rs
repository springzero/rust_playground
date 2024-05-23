use kml::KmlDocument;
use kml::{types::Point, Kml, KmlWriter};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Error as SerError;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::{fs::File, io::Read};
use tera::{to_value, try_get_value, Context, Tera, Value};

#[derive(Debug, Deserialize, Serialize)]
struct Gateway {
    pub name: String,
    pub swver: String,
    pub gps: Option<String>,
    pub ip: Option<String>,
    pub hoplist: Vec<u16>,
    pub txpower: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct MyPoint {
    pub long: f64,
    pub lati: f64,
    pub name: String,
}

fn extract_coordinates(s: &str) -> Option<(f64, f64)> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"lat=(-?\d+\.\d+) N\s*,\s*lng=(-?\d+\.\d+) E"#).unwrap();
    }
    let caps = RE.captures(s)?;
    let lat = caps.get(1)?.as_str().parse::<f64>().ok()?;
    let lng = caps.get(2)?.as_str().parse::<f64>().ok()?;

    Some((lat, lng))
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["kml"]);

        tera
    };
}

// fn vec_u16_to_string(mut vec: Vec<u16>) -> String {
//     let mut result = String::new();
//     for num in vec.iter() {
//         write!(&mut result, "{:_}", num).unwrap();
//         if num != vec.last().unwrap() {
//             result.push('_');
//         }
//     }
//     result
// }

fn main() {
    let mut file = File::open("input.json").expect("Failed to open file");
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)
        .expect("Failed to read file");

    let gws: Result<Vec<Gateway>, SerError> = serde_json::from_reader(buffer.as_bytes());

    let mut points = vec![];
    match gws {
        Ok(gws) => {
            for gw in gws.into_iter() {
                match gw.gps {
                    Some(gps) => {
                        if let Some((lat, lng)) = extract_coordinates(&gps) {
                            let hoplist: String = gw
                                .hoplist
                                .iter()
                                .map(|&x| x.to_string())
                                .collect::<Vec<String>>()
                                .join("|");
                            let name = format!("{}_{}_{}", gw.name, hoplist, gw.txpower);
                            let p = MyPoint {
                                long: lng / 100.0,
                                lati: lat / 100.0,
                                name: name,
                            };
                            points.push(p);
                        } else {
                            println!(
                                "Could not extract coordinates from the given string {:?}",
                                gw.name
                            );
                        }
                    }
                    None => {}
                }
            }
        }
        Err(e) => {
            println!("gws_error:{:?}", e);
        }
    }

    let mut context = Context::new();
    context.insert("points", &points);

    match TEMPLATES.render("temp.kml", &context) {
        Ok(s) => {
            let mut file = File::create("output.kml").unwrap();
            file.write_all(s.as_bytes());
            file.flush();
        }
        Err(e) => {
            println!("Error: {}", e);
            let mut cause = e.source();
            while let Some(e) = cause {
                println!("Reason: {}", e);
                cause = e.source();
            }
        }
    };
}

#[test]
fn test_regx() -> () {
    let s = "lat=4456.26233 N ,lng=08902.03126 E";
    if let Some((lat, lng)) = extract_coordinates(s) {
        println!("Latitude: {}, Longitude: {}", lat, lng);
    } else {
        println!("Could not extract coordinates from the given string");
    }
}

#[test]
fn test_one() -> () {
    let vec: Vec<u16> = vec![1, 2, 3];
    let str: String = vec
        .iter()
        .map(|&x| x.to_string())
        .collect::<Vec<String>>()
        .join("_");
    println!("{}", str);
}
