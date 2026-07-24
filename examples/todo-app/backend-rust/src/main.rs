mod application;
mod domain;
mod infrastructure;
mod interface;

use std::path::PathBuf;

use application::todo_service::TodoService;
use infrastructure::sqlite::TodoRepository;
use interface::http::{router, AppState};

#[tokio::main]
async fn main() {
    let mut host = "127.0.0.1".to_owned();
    let mut port = 8080_u16;
    let mut database = PathBuf::from("data/todos.sqlite3");
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--host" => host = arguments.next().expect("--host requires a value"),
            "--port" => port = arguments.next().expect("--port requires a value").parse().expect("--port must be a number"),
            "--db" => database = arguments.next().expect("--db requires a value").into(),
            "--help" | "-h" => { println!("focusflow-todo [--host HOST] [--port PORT] [--db PATH]"); return; }
            _ => panic!("unknown argument: {argument}"),
        }
    }
    let repository = TodoRepository::open(database).expect("initialize SQLite database");
    let address = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&address).await.expect("bind HTTP listener");
    println!("FocusFlow is ready at http://{address}");
    axum::serve(listener, router(AppState { todos: TodoService::new(repository) })).await.expect("serve HTTP API");
}
