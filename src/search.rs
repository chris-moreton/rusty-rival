use std::process::exit;
use std::sync::mpsc::Sender;
use std::time::Instant;

pub fn search(tx: Sender<String>) {
    let mut i = 0;
    loop {
        i += 1;
        let val = String::from(i.to_string());
        tx.send(val).unwrap();
    }

    let val = String::from("done");
    tx.send(val).unwrap();

}