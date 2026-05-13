use crossterm::{
    ExecutableCommand,
    cursor::{EnableBlinking, Hide},
    event::{self, Event, KeyCode, poll},
    style::{Attribute, Color, Stylize},
    terminal::{Clear, disable_raw_mode, enable_raw_mode},
};
use local_ip_address::local_ip;
use netshare::*;
use std::io::{Result, Write, stdout};
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

    let menu_items = vec!["Connect", "Quit"];
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
        "Connect" => {
            // there are exactly 2 ways we get to start_chat()
            // 1. somebody connects to us.
            // 2. we connect to someone else
            //
            // do whatever happens first

            let tcp_listen_future = listen_tcp(&my_ipv4);
            let tcp_establish_future = connect(&my_ipv4);

            let res_tcp_stream = tokio::select! {
                result = tcp_listen_future => result.unwrap(),
                result = tcp_establish_future => result.unwrap(),
            };

            start_chat(res_tcp_stream).await;
        }
        "Quit" => {
            println!("Exiting...");
        }
        _ => panic!("Bad input.\n"),
    }

    std::process::exit(0);
}
