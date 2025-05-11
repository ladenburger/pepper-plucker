use pepper_plucker::configuration::get_configuration;
use pepper_plucker::startup::Application;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await;

    match application {
        Ok(app) => app.run_until_stopped().await,
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Application build failed. {:?}", e),
        )),
    }
}
