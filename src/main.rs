/*
RECEIVER:
- Listen for UDP connections at port 14953.
- You will get a message asking for your ip.
- Respond with ip via UDP.
- After this the sender will estabish a tcp connection
- The sender will send you a message
- Chat.
- Fin.

SENDER:
- Send the prompt message to the network broadcast address *DONE*
- If you get a response, it will be the remote machine's ip address *DONE*
- Establish a tcp connection with that machine. *DONE*
- Send messages back and forth with the remote machine *DONE?*.


*/

use netshare::*;
use std::io::{self, Result, Write};
use local_ip_address::local_ip;
use std::net::{ Ipv4Addr };

fn main() -> Result<()>
{
    // what's my IP
    let my_ipv4 : Ipv4Addr = to_ipv4(local_ip().unwrap())
        .unwrap();

    let mut mode = String::new();

    println!("
        ███╗   ██╗ ███████╗ ████████╗ ███████╗ ███████╗ ███╗   ██╗ ██████╗ 
        ████╗  ██║ ██╔════╝ ╚══██╔══╝ ██╔════╝ ██╔════╝ ████╗  ██║ ██╔══██╗
        ██╔██╗ ██║ █████╗      ██║    ███████╗ █████╗   ██╔██╗ ██║ ██║  ██║
        ██║╚██╗██║ ██╔══╝      ██║    ╚════██║ ██╔══╝   ██║╚██╗██║ ██║  ██║
        ██║ ╚████║ ███████╗    ██║    ███████║ ███████╗ ██║ ╚████║ ██████╔╝
        ╚═╝  ╚═══╝ ╚══════╝    ╚═╝    ╚══════╝ ╚══════╝ ╚═╝  ╚═══╝ ╚═════╝ ");

    print!("\n\nEnter 'listen' for listening mode, or 'send' for sending mode: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut mode)
        .expect("Failed to read line");

    match mode.trim() 
    {
        "listen" => 
        {
            println!("Entering listening mode...");
            receive(&my_ipv4)?;

        },
        "send" => 
        {
            println!("Entering sending mode...");
            sender(&my_ipv4);
        },
        _ => panic!("Bad input.\n"),
    }

    Ok(())
}
