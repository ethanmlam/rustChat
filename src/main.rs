// 1. CLI
// 2. Receive/Send messages using stdin/stdout
// 3. Use terminal raw mode to fix some bugs
//
//

use std::io::{BufRead, BufReader, Write};
use std::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use clap::{Parser, Subcommand, arg, command};
use colored::Colorize;
use crossterm::{
    event::{self, Event as TermEvent, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    execute,
};

#[derive(Parser)]
#[command(name = "chat")]
#[command(about = "A simple TCP chat application", long_about = None)]
struct Cli {
    /// Your display name
    #[arg(value_name = "NAME")]
    name: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Connect to a server
    Connect {
        /// IP address to connect to
        #[arg(value_name = "HOST")]
        ip: String,

        /// Port number to connect to
        #[arg(value_name = "PORT")]
        port: u16,
    },
    /// Start a server
    Start {
        /// IP address to bind to
        #[arg(value_name = "HOST")]
        ip: String,

        /// Port number to bind to
        #[arg(value_name = "PORT")]
        port: u16,
    },
}

enum Event {
    RecvMessage(String),
    SendMessage(String),
    Exit,
}

fn get_formatted_time() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn print_with_time(msg: &str) {
    let time_str = get_formatted_time();
    print!("[{}] {}\n", time_str.dimmed(), msg);
    std::io::Write::flush(&mut std::io::stdout()).expect("failed to flush stdout");
}

fn run_event_loop(name: String, mut stream: TcpStream, recv: mpsc::Receiver<Event>) {
    let mut stdout = std::io::stdout();

    loop {
        match recv.recv().expect("failed to receive event") {
            Event::Exit => {
                print_with_time("exiting...");
                break;
            }
            Event::RecvMessage(msg) => {
                // Clear current line before printing received message
                print!("\r");
                execute!(stdout, Clear(ClearType::CurrentLine)).expect("failed to clear line");

                print_with_time(&msg);

                // Move cursor to far left for next input
                print!("\r");
                stdout.flush().expect("failed to flush stdout");
            }
            Event::SendMessage(msg) => {
                stream
                    .write_all(format!("{}: {}\n", name.blue(), msg).as_bytes())
                    .expect("failed to send message");

                print_with_time(&format!("{}: {}", name.green(), msg));

                // Move cursor to far left for next input
                print!("\r");
                stdout.flush().expect("failed to flush stdout");
            }
        }
    }
}

/// Listen on addr:port, accept first connection
/// and return the TcpStream
fn start(addr: String, port: u16) -> TcpStream {
    print!("\r");
    println!("listening on {}:{}", addr, port);

    let listener = TcpListener::bind((addr, port)).expect("failed to bind to address");

    let (stream, client_addr) = listener.accept().expect("failed to accept connection");
    print!("\r");
    println!("client connected from {}", client_addr);

    stream
}

/// Connect to addr:port and return the TcpStream
fn connect(addr: String, port: u16) -> TcpStream {
    print!("\r");
    println!("connecting to {}:{}", addr, port);

    let stream = TcpStream::connect((addr.clone(), port)).expect("failed to connect to server");
    print!("\r");
    println!("connected to server at {}:{}", &addr, port);

    stream
}

fn start_message_listener(stream: TcpStream, event_tx: mpsc::Sender<Event>) {
    std::thread::spawn(move || {
        let reader = BufReader::new(stream);
        for msg in reader.lines() {
            let msg = msg.expect("failed to read message from stream");

            event_tx
                .send(Event::RecvMessage(msg))
                .expect("failed to send received message event");
        }

        event_tx
            .send(Event::Exit)
            .expect("failed to send exit event");
    });
}

fn start_input_listener(event_tx: mpsc::Sender<Event>) {
    std::thread::spawn(move || {
        enable_raw_mode().expect("failed to enable raw mode");

        let mut buffer = String::new();
        let mut stdout = std::io::stdout();

        loop {
            if let Ok(TermEvent::Key(KeyEvent { code, modifiers, .. })) = event::read() {
                match code {
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Char(c) => {
                        buffer.push(c);
                        print!("{}", c);
                        stdout.flush().expect("failed to flush stdout");
                    }
                    KeyCode::Backspace => {
                        if buffer.pop().is_some() {
                            print!("\x08 \x08");
                            stdout.flush().expect("failed to flush stdout");
                        }
                    }
                    KeyCode::Enter => {
                        if buffer == "exit" {
                            println!();
                            break;
                        }
                        if !buffer.is_empty() {
                            // Clear the typed input line, cursor stays at line start
                            print!("\r");
                            execute!(stdout, Clear(ClearType::CurrentLine)).expect("failed to clear line");
                            stdout.flush().expect("failed to flush stdout");

                            event_tx
                                .send(Event::SendMessage(buffer.clone()))
                                .expect("failed to send send message event");
                            buffer.clear();
                        } else {
                            println!();
                        }
                    }
                    _ => {}
                }
            }
        }

        disable_raw_mode().expect("failed to disable raw mode");
        event_tx
            .send(Event::Exit)
            .expect("failed to send exit event");
    });
}

fn main() {
    let cli = Cli::parse();
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let name = cli.name;
    let stream = match cli.command {
        Commands::Connect { ip, port } => connect(ip, port),
        Commands::Start { ip, port } => start(ip, port),
    };

    start_message_listener(
        stream.try_clone().expect("failed to clone"),
        event_tx.clone(),
    );
    start_input_listener(event_tx);

    run_event_loop(name, stream, event_rx);
}