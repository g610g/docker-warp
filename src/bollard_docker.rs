use crate::ws::Clients;
use bollard::{
    container::{ListContainersOptions, LogOutput, LogsOptions},
    secret::ContainerSummary,
    Docker,
};
use std::{error, str::from_utf8};
use tokio_stream::StreamExt;
use warp::filters::ws::Message;

pub async fn initialize(clients: Clients) -> Result<(), Box<dyn error::Error>> {
    let docker_interface = Docker::connect_with_unix_defaults()?;
    // let mut handlers = vec![];
    let container_summary_vec = docker_interface
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await?;

    for container_summary in container_summary_vec {
        let clients_clone = clients.clone();
        let docker_interface_clone = docker_interface.clone();
        let handler = tokio::spawn(async move {
            get_logs(docker_interface_clone, container_summary, clients_clone)
                .await
                .unwrap();
        });
        // handlers.push(handler);
    }
    // for handler in handlers {
    //     let _ = handler.await;
    // }
    Ok(())
}

async fn get_logs(
    docker_interface: Docker,
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
                LogOutput::StdOut { message } => {
                    let locked = clients.lock().await;
                    for (_, val) in locked.iter() {
                        if let Some(sender) = &val.sender {
                            let string_message = from_utf8(&message)?;
                            println!("Logs");
                            let _ = sender.send(Message::text(string_message));
                        }
                    }
                }
                _ => println!("Idk what to do for the console"),
            }
        }
    }
    Ok(())
}
