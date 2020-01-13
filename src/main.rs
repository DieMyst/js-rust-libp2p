use server::serve;

mod server;
mod behaviour;

fn main() {
    env_logger::init();
    serve(30000)
}
