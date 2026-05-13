use crossterm::{
    ExecutableCommand,
    cursor::{EnableBlinking, Hide},
    event::{self, Event, KeyCode, poll},
    style::{Attribute, Color, Stylize},
    terminal::{Clear, disable_raw_mode, enable_raw_mode},
};
use local_ip_address::local_ip;
use netshare::*;
use std::{io::{Result, Write, stdout}};
use std::net::Ipv4Addr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // what's my IP
    let my_ipv4: Ipv4Addr = to_ipv4(local_ip().unwrap()).unwrap();

    enable_raw_mode()?;
    stdout().execute(Clear(crossterm::terminal::ClearType::All))?;
    stdout().execute(Hide)?;

    // Drain any buffered input before entering the menu
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read();
    }

    let logo = "NETSHARE"
        .with(Color::Yellow)
        .on(Color::Blue)
        .attribute(Attribute::Bold);

    let menu_items = vec![
        "Connect", 
        "Quit"
    ];
    let mut selection = 0;

    stdout().execute(crossterm::cursor::MoveTo(0, 0))?;
    stdout().write_all(format!("{}\r\n", logo).as_bytes())?;

    // this is for windows powershell, does not work without it.
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read();
    }

    loop {
        stdout().execute(crossterm::cursor::MoveTo(0, 2))?;

        for (i, item) in menu_items.iter().enumerate() {
            if i == selection {
                stdout().write_all(b"> ")?;
            } else {
                stdout().write_all(b"  ")?;
            }
            stdout().write_all(item.as_bytes())?;
            stdout().write_all(b"\r\n")?;
        }
        stdout().flush()?;

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != event::KeyEventKind::Press {
                    continue;
                }
                match key_event.code {
                    KeyCode::Char('k') | KeyCode::Up if selection > 0 => selection -= 1,
                    KeyCode::Char('j') | KeyCode::Down if selection < menu_items.len() - 1 => {
                        selection += 1
                    }
                    KeyCode::Enter => break,
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    stdout().execute(EnableBlinking)?;
    stdout().execute(Hide)?;
    disable_raw_mode()?;

    match menu_items[selection] {
        "Listen" => {
            println!("Entering listening mode...");
            let tcp_stream = receive(&my_ipv4).await?;
            start_chat(tcp_stream).await;
        }
        "Connect" => {
            println!("Sending and Receiving at the same time...");

            let recv_future = receive(&my_ipv4);
            let send_future = sender(&my_ipv4);

            tokio::select! {
                tcp_stream = recv_future => {
                    println!("recv finished first");
                    start_chat(tcp_stream.unwrap()).await;
                },
                tcp_stream = send_future => {
                    println!("send finished first");
                    start_chat(tcp_stream.unwrap()).await;
                },
            };
        }
        _ => panic!("Bad input.\n"),
    }

    std::process::exit(0);
}
