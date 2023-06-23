use std:: { net::TcpListener, sync::{ Arc, Mutex } };
use vr_lib::*;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize)]
struct Board {
    board: [Values; 9],
    turn: Values,
}

#[derive(Debug, Serialize, Deserialize)]
struct Move {
    index: usize,
    id: Values
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum Values {
    X,
    O,
    Empty,
}

fn main() {

    let state = Arc::new(Mutex::new(
        Board { board: [Values::Empty; 9], turn: Values::X }
    ));

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let state = state.clone();
        let stream = stream.unwrap();

        // handle_connection(state.clone(), stream);
        let mut connection: Connection<Board> = Connection::new(stream);
        connection
            .mount_state(state)
            .mount_handlers(vec![index, post_move])
            .serve();
    }
}


fn index(con: &Connection<Board>) -> Response {
    if con.req.request_line.as_str() != "GET / HTTP/1.1" {
        return Response::empty();
    }

    let state = con.state.clone().unwrap();

    let mut res = Response { status_line: "HTTP/1.1 400 BAD REQUEST".to_string(), headers: Vec::new(), body: Vec::new() };
    if let Ok(mutex) = state.try_lock() {
        res.status_line = "HTTP/1.1 200 OK".to_string();
        res.headers.push(("Content-Type".to_string(), "application/json".to_string()));
        res.body =  serde_json::to_string(&*mutex).unwrap().as_bytes().to_vec();
    }
    res
}

fn post_move(con: &Connection<Board>) -> Response {
    if con.req.request_line.as_str() != "POST /move HTTP/1.1" {
        return Response::empty();
    }

    let state = con.state.clone().unwrap();
    let mut board = state.lock().unwrap();
    let move_: Move = serde_json::from_str(std::str::from_utf8(con.req.body.as_slice()).unwrap()).unwrap();

    if board.turn != move_.id || board.board[move_.index] != Values::Empty {
        return Response { status_line: "HTTP/1.1 400 BAD REQUEST".to_string(), headers: Vec::new(), body: Vec::new() };
    }

    board.board[move_.index] = move_.id;
    board.turn = if move_.id == Values::X { Values::O } else { Values::X };

    Response {
        status_line: "HTTP/1.1 200 OK".to_string(),
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        body: serde_json::to_string(&*board).unwrap().as_bytes().to_vec()
    }
}
