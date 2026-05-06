use miniserde::{json, Serialize};
use std::io::{Write, Read};
use std::net::{TcpStream, UdpSocket, ToSocketAddrs};
use std::time::Duration;

#[derive(Serialize)]
struct TestResult<'a> {
    test_type: &'a str,
    success: bool,
    message: String,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wasi-network <test_type> [args...]");
        eprintln!("  test_type: tcp, udp, dns");
        std::process::exit(1);
    }

    let test_type = &args[1];
    let result = match test_type.as_str() {
        "tcp" => test_tcp(&args),
        "udp" => test_udp(&args),
        "dns" => test_dns(&args),
        _ => TestResult {
            test_type: test_type.as_str(),
            success: false,
            message: format!("Unknown test type: {}", test_type),
        },
    };

    std::io::stdout().write_all(json::to_string(&result).as_bytes())
        .expect("failed to write to stdout");
}

fn test_tcp(args: &[String]) -> TestResult<'static> {
    if args.len() < 4 {
        return TestResult {
            test_type: "tcp",
            success: false,
            message: "Usage: wasi-network tcp <host> <port>".to_string(),
        };
    }

    let host = &args[2];
    let port = &args[3];
    let addr = format!("{}:{}", host, port);

    match TcpStream::connect_timeout(
        &addr.parse().expect("failed to parse address"),
        Duration::from_secs(2)
    ) {
        Ok(mut stream) => {
            // Try to send and receive some data
            match stream.write_all(b"HELLO") {
                Ok(_) => {
                    let mut buf = [0u8; 1024];
                    match stream.read(&mut buf) {
                        Ok(n) if n > 0 => {
                            TestResult {
                                test_type: "tcp",
                                success: true,
                                message: format!("Connected to {} and exchanged data", addr),
                            }
                        }
                        Ok(_) => {
                            TestResult {
                                test_type: "tcp",
                                success: true,
                                message: format!("Connected to {} (no response)", addr),
                            }
                        }
                        Err(e) => {
                            TestResult {
                                test_type: "tcp",
                                success: false,
                                message: format!("Failed to read from {}: {}", addr, e),
                            }
                        }
                    }
                }
                Err(e) => {
                    TestResult {
                        test_type: "tcp",
                        success: false,
                        message: format!("Failed to write to {}: {}", addr, e),
                    }
                }
            }
        }
        Err(e) => {
            TestResult {
                test_type: "tcp",
                success: false,
                message: format!("Failed to connect to {}: {}", addr, e),
            }
        }
    }
}

fn test_udp(args: &[String]) -> TestResult<'static> {
    if args.len() < 4 {
        return TestResult {
            test_type: "udp",
            success: false,
            message: "Usage: wasi-network udp <host> <port>".to_string(),
        };
    }

    let host = &args[2];
    let port = &args[3];
    let addr = format!("{}:{}", host, port);

    match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => {
            match socket.send_to(b"HELLO", &addr) {
                Ok(_) => {
                    let mut buf = [0u8; 1024];
                    match socket.recv_from(&mut buf) {
                        Ok((n, _)) if n > 0 => {
                            TestResult {
                                test_type: "udp",
                                success: true,
                                message: format!("Sent to {} and received response", addr),
                            }
                        }
                        Ok(_) => {
                            TestResult {
                                test_type: "udp",
                                success: true,
                                message: format!("Sent to {} (no response)", addr),
                            }
                        }
                        Err(e) => {
                            TestResult {
                                test_type: "udp",
                                success: false,
                                message: format!("Failed to receive from {}: {}", addr, e),
                            }
                        }
                    }
                }
                Err(e) => {
                    TestResult {
                        test_type: "udp",
                        success: false,
                        message: format!("Failed to send to {}: {}", addr, e),
                    }
                }
            }
        }
        Err(e) => {
            TestResult {
                test_type: "udp",
                success: false,
                message: format!("Failed to bind UDP socket: {}", e),
            }
        }
    }
}

fn test_dns(args: &[String]) -> TestResult<'static> {
    if args.len() < 3 {
        return TestResult {
            test_type: "dns",
            success: false,
            message: "Usage: wasi-network dns <hostname>".to_string(),
        };
    }

    let hostname = &args[2];
    let addr_with_port = format!("{}:80", hostname);

    match addr_with_port.to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                TestResult {
                    test_type: "dns",
                    success: true,
                    message: format!("Resolved {} to {}", hostname, addr.ip()),
                }
            } else {
                TestResult {
                    test_type: "dns",
                    success: false,
                    message: format!("No addresses found for {}", hostname),
                }
            }
        }
        Err(e) => {
            TestResult {
                test_type: "dns",
                success: false,
                message: format!("Failed to resolve {}: {}", hostname, e),
            }
        }
    }
}
