mod regex;
mod statemachine;
mod transforms;

use self::statemachine::*;
use self::transforms::*;

fn main() {
    let mut sm = StateMachine::from_seq(b"FLAG{");
    regex::meme();
    // StateMachine::from_regex(r"FLAG\{\w{31}=\}")
    ignore_case(&mut sm);
    sm.union(hexlify(&sm));
    ignore_case(&mut sm);
    println!("{:#?}", sm);
    for m in sm.matches(
        b"dank flag{ memes } 666c61677b20617979207d0a"
            .iter()
            .cloned(),
    ) {
        println!("{:?}", String::from_utf8_lossy(&m));
    }
}
