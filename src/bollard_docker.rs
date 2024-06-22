use bollard::{
    container::{ListContainersOptions, LogOutput, LogsOptions},
    secret::ContainerSummary,
    Docker,
};
use std::error;
use tokio_stream::StreamExt;

use crate::ws::Clients;
pub async fn run_with_bollard() -> Result<(), Box<dyn error::Error>> {
    let docker_interface = Docker::connect_with_unix_defaults()?;
    let container_summary_vec = docker_interface
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;
    let mut handlers = vec![];
    for container_summary in container_summary_vec {
        let docker_interface_clone = docker_interface.clone();
        let handler = tokio::spawn(async move {
            get_logs(&docker_interface_clone, container_summary)
                .await
                .unwrap();
        });
        handlers.push(handler);
    }
    for handler in handlers {
        let _ = handler.await;
    }
    Ok(())
}

async fn get_logs(
    docker_interface: &Docker,
    container_summary: ContainerSummary,
    clients: Clients,
) -> Result<(), Box<dyn error::Error>> {
    let container_id = match container_summary.id {
        Some(id) => id,
        None => {
            return Err("We returned err when there is no name for the container".into());
        }
    };
    let mut stream = docker_interface.logs(
        &container_id,
        Some(LogsOptions::<String> {
            follow: true,
            stdout: true,
            timestamps: true,
            ..Default::default()
        }),
    );
    while let Some(data) = stream.next().await {
        if let Ok(logoutput) = data {
            match logoutput {
                LogOutput::StdErr { message } => {
                    eprintln!("Bytes we recieve stderr: {:?}", message);
                }
                LogOutput::StdOut { message } => {
                    //this is the thing we want
                    println!("Bytes we recieve from stdout: {:?}", message);
                }
                LogOutput::StdIn { message } => {
                    println!("Bytes we recieve from stdin: {:?}", message);
                }
                _ => println!("Idk what to do for the console"),
            }
        }
    }
    Ok(())
}
