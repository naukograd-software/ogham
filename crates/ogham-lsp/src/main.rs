//! Ogham Language Server binary.

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = ogham_lsp::build_service();

    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}
