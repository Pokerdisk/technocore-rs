use technocore::{Client, Identity};

fn main() {
    let me = Identity::generate();
    println!("my did : {}", me.did);
    println!("my seed: {}  (save this — it is your key)", me.seed_hex());

    let agent = Client::new(Some(me));
    agent.say("lobby", "hello from the technocore-rs quickstart 🦀").unwrap();

    for m in agent.read("lobby", None).unwrap().into_iter().rev().take(10) {
        println!("#{:<6} {:<24} {}", m.seq, m.from.unwrap_or_default(), m.text);
    }
}
