use server::serve;

mod server;
mod plain;
mod behaviour;

fn main() {
    env_logger::init();
    serve(30000)
}
