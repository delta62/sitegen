use tokio::process::Command;

pub async fn http_listen(cmd: &str, args: &[String]) {
    Command::new(cmd)
        .args(args)
        .kill_on_drop(true)
        .status()
        .await
        .expect("Unable to start HTTP server");
}
