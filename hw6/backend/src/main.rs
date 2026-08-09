use backend::{create_app, default_state};

#[tokio::main]
async fn main() {
    let state = default_state();
    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Не удалось привязать порт 3000");

    println!("Сервер запущен на http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("Ошибка сервера");
}
