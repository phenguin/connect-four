#![feature(use_extern_macros)]

extern crate gameai;
extern crate bincode;
extern crate clap;

use clap::{Arg, App, value_t};
use std::io::{Read, Write};

use gameai::game;
use std::net::TcpStream;
use bincode::{deserialize, serialize, Infinite};
use gameai::game::ParseGame;

fn do_main(mut conn: TcpStream) {
    loop {
        let mut buf = [0; 512];
        println!("Waiting for input.");
        conn.read(&mut buf).unwrap();
        let game: game::connectfour::ConnectFour = deserialize(&buf[..]).unwrap();
        println!("{}", game);

        loop {
            println!("What is your move?");
            let mut choice = String::new();
            std::io::stdin().read_line(&mut choice).expect(
                "Failed to read line... something is mad broke.",
            );
            println!("");

            let choice = match game.parse_move(choice.trim()) {
                Some(m) => m,
                None => continue,
            };

            println!("{:?}", choice);

            // if !game.move_valid(&choice) {
            //     println!("Invalid move..");
            //     continue;
            // }


            conn.write(&serialize(&choice, Infinite).unwrap()).unwrap();
            break;
        }
    }
}

fn main() {
    let matches = App::new("Connect Four")
        .version("0.1.0")
        .about("Simple project to play with while learning rust")
        .arg(
            Arg::with_name("server_addr")
                .short("s")
                .value_name("STRING")
                .long("server_address")
                .help(
                    "<hostname>::port for the server",
                )
                .takes_value(true),
        )
        .get_matches();

    let server_addr = value_t!(matches.value_of("server_addr"), String).unwrap_or_else(|e| e.exit());
    do_main(TcpStream::connect(server_addr).unwrap());
}
