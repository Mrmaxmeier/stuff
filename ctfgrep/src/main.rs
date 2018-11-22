mod regex;
mod statemachine;
mod transforms;

use self::statemachine::*;
use self::transforms::*;

const TEST_INPUT: &[u8; 106] = b"ctf{dank} flag{memes} 666c61677b20617979207d0a Q1RGe21lbWVzfQo= QUNURnt4fQo= QUFDVEZ7eX0K QUFBQ1RGe3p9Cg==";

fn main() {
    // let mut sm = regex::SMBuilder::construct_statemachine(r"FLAG\{\w+\}").unwrap();
    let _tmp = regex::SMBuilder::construct_statemachine(r"\d*").unwrap();
    println!("{:?}", _tmp);
    _tmp.dump_dot();
    let _tmp = hexlify(&_tmp);
    println!("{:?}", _tmp);
    _tmp.dump_dot();

    let mut _tmp = StateMachine::from_seq(b"X");
    _tmp.concat(StateMachine::from_seq(b"Y"));
    _tmp.dump_dot();
    ignore_case(&mut _tmp);
    _tmp.dump_dot();

    let mut sm = regex::SMBuilder::construct_statemachine(r"(CTF|FLAG)\{\w+\}").unwrap();
    println!("meems {:#?}", sm);
    sm.dump_dot();
    ignore_case(&mut sm);
    sm.dump_dot();
    hexlify(&sm).dump_dot();
    sm.union(hexlify(&sm));
    sm.dump_dot();
    ignore_case(&mut sm);
    println!("{:#?}", sm);
    sm.dump_dot();
    for m in sm.matches(TEST_INPUT.iter().cloned()) {
        println!("{:?}", String::from_utf8_lossy(&m));
    }
}
