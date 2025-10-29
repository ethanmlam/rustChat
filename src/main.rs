// 1. CLI
// 2. Receive/Send messages using stdin/stdout
// 3. Use terminal raw mode to fix some bugs
//
//

mod certs;

use std::io::Write;
use std::sync::Arc;

use clap::{Parser, Subcommand, arg, command};
use colored::Colorize;
use crossterm::{
    cursor,
    event::{self, Event as TermEvent, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    execute,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_rustls::{TlsAcceptor, TlsConnector, TlsStream};
use rustls::pki_types::ServerName;

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

async fn run_event_loop<W>(
    name: String,
    mut writer: W,
    recv: &mut mpsc::Receiver<Event>,
) where
    W: tokio::io::AsyncWrite + Unpin,
{
    let mut stdout = std::io::stdout();

    loop {
        match recv.recv().await {
            Some(Event::Exit) => {
                print_with_time("exiting...");
                break;
            }
            Some(Event::RecvMessage(msg)) => {
                // Clear current line before printing received message
                print!("\r");
                execute!(stdout, Clear(ClearType::CurrentLine)).expect("failed to clear line");

                print_with_time(&msg);

                // Move cursor to far left for next input
                print!("\r");
                stdout.flush().expect("failed to flush stdout");
            }
            Some(Event::SendMessage(msg)) => {
                writer
                    .write_all(format!("{}: {}\n", name.blue(), msg).as_bytes())
                    .await
                    .expect("failed to send message");

                print_with_time(&format!("{}: {}", name.green(), msg));

                // Move cursor to far left for next input
                print!("\r");
                stdout.flush().expect("failed to flush stdout");
            }
            None => {
                print_with_time("channel closed, exiting...");
                break;
            }
        }
    }
}

/// Listen on addr:port, accept first connection with TLS
/// and return the TLS stream
async fn start(addr: String, port: u16) -> TlsStream<TcpStream> {
    let mut stdout = std::io::stdout();

    // Clear screen and move cursor to top-left
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))
        .expect("failed to clear screen");

    println!("🚀 Starting chat server with TLS...");
    println!("listening on {}:{}", addr, port);

    // Load certificates and configure TLS
    let certs = certs::load_certs().expect("failed to load certificates");
    let key = certs::load_private_key().expect("failed to load private key");

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("failed to create TLS config");

    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind((addr, port))
        .await
        .expect("failed to bind to address");

    let (tcp_stream, client_addr) = listener
        .accept()
        .await
        .expect("failed to accept connection");

    println!("✓ client connected from {}, performing TLS handshake...", client_addr);

    let tls_stream = acceptor
        .accept(tcp_stream)
        .await
        .expect("failed to complete TLS handshake");

    println!("✓ TLS handshake complete - connection is encrypted");
    println!("\nChat started! Type your messages and press Enter to send.");
    println!("Type 'exit' or press Ctrl+C to quit.\n");

    TlsStream::Server(tls_stream)
}

/// Custom certificate verifier that accepts self-signed certificates
#[derive(Debug)]
struct NoCertVerifier;

impl rustls::client::danger::ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // Accept any certificate (for self-signed certs in private chat)
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

/// Connect to addr:port with TLS and return the TLS stream
async fn connect(addr: String, port: u16) -> TlsStream<TcpStream> {
    let mut stdout = std::io::stdout();

    // Clear screen and move cursor to top-left
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))
        .expect("failed to clear screen");

    println!("🚀 Connecting to chat server with TLS...");
    println!("connecting to {}:{}", addr, port);

    // Configure TLS client - accept self-signed certificates
    // For self-signed certs, we need a custom verifier that accepts them
    let config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoCertVerifier))
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));

    let tcp_stream = TcpStream::connect((addr.clone(), port))
        .await
        .expect("failed to connect to server");

    println!("✓ TCP connected, performing TLS handshake...");

    let server_name = ServerName::try_from("localhost")
        .expect("invalid DNS name");

    let tls_stream = connector
        .connect(server_name, tcp_stream)
        .await
        .expect("failed to complete TLS handshake");

    println!("✓ TLS handshake complete - connection is encrypted");
    println!("\nChat started! Type your messages and press Enter to send.");
    println!("Type 'exit' or press Ctrl+C to quit.\n");

    TlsStream::Client(tls_stream)
}

async fn start_message_listener<R>(
    reader: R,
    event_tx: mpsc::Sender<Event>,
) where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut reader = AsyncBufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // Connection closed
                let _ = event_tx.send(Event::Exit).await;
                break;
            }
            Ok(_) => {
                let msg = line.trim_end().to_string();
                if event_tx.send(Event::RecvMessage(msg)).await.is_err() {
                    break;
                }
            }
            Err(_) => {
                let _ = event_tx.send(Event::Exit).await;
                break;
            }
        }
    }
}

fn start_input_listener(event_tx: mpsc::Sender<Event>) {
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
                        execute!(stdout, Clear(ClearType::CurrentLine))
                            .expect("failed to clear line");
                        stdout.flush().expect("failed to flush stdout");

                        event_tx
                            .blocking_send(Event::SendMessage(buffer.clone()))
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
        .blocking_send(Event::Exit)
        .expect("failed to send exit event");
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let (event_tx, mut event_rx) = mpsc::channel::<Event>(100);

    let name = cli.name;
    let stream = match cli.command {
        Commands::Connect { ip, port } => connect(ip, port).await,
        Commands::Start { ip, port } => start(ip, port).await,
    };

    let (reader, writer) = tokio::io::split(stream);

    tokio::spawn(start_message_listener(reader, event_tx.clone()));

    tokio::task::spawn_blocking(move || {
        start_input_listener(event_tx);
    });

    run_event_loop(name, writer, &mut event_rx).await;
}