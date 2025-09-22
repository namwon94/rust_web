use std::net::TcpListener;
use rust_web::{
    configuration::get_configuration,
    startup::run,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 구성을 읽을 수 없으면 패닉에 빠진다
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener)?.await
}



/*
fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    //incoming 메서드 : 스트림 시쿼스(TcpStream 타입의 스트림)를 제공하는 반복자를 반환. 스트림이란 클라이언트와 서버 간의 개방형 연결을 나타낸다.
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        habdle_connection(stream);
    }
}

fn habdle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();   
    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    }else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}
*/
