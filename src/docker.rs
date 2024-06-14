use docker_api::{
    self,
    conn::TtyChunk,
    models::ContainerSummary,
    opts::{ContainerListOpts, LogsOpts},
    Container, Containers,
};
use futures::StreamExt;
pub async fn docker() {
    let docker = docker_api::Docker::unix("/var/run/docker.sock");
    let containers = docker.containers();
    let containers_list = give_me_containers(&containers).await;
    //constructs a LogsOpts from LogsOpts Builder
    let logopts = LogsOpts::builder()
        .all()
        .stdout(true)
        .stderr(true)
        .timestamps(true)
        .build();
    let mut handlers = vec![];
    //asynchronously waiting for stream of logs from containers
    //spawn a tokio task for a nonblocking waiting of logs
    for container in containers_list {
        let logopts_clone = logopts.clone();
        let handler = tokio::spawn(async move {
            while let Some(chunk) = container.logs(&logopts_clone).next().await {
                match chunk {
                    Ok(chunk) => {
                        //we want to convert the u8 bytes into its utf8 string representation
                        let str = bytes_to_string(chunk);
                        println!(
                            "From container: {} We got: {}",
                            container.id().to_string(),
                            str
                        );
                    }
                    Err(e) => eprintln!("We got error from stream log: {e}"),
                }
            }
        });
        handlers.push(handler);
    }
    for handler in handlers {
        let _ = handler.await;
    }
}
async fn give_me_containers(container_interface: &Containers) -> Vec<Container> {
    let mut result_containers = vec![];
    let container_summaries = container_lists(&container_interface).await;
    for container_summary in container_summaries {
        match container_summary.id {
            Some(id) => result_containers.push(container_interface.get(id)),
            None => println!("No id for this container"),
        };
    }
    return result_containers;
}
async fn container_lists(containers_interface: &Containers) -> Vec<ContainerSummary> {
    let builder = ContainerListOpts::builder();
    let container_list_opts = builder.all(true).build();
    containers_interface
        .list(&container_list_opts)
        .await
        .unwrap()
}

//deref the TtyChunk into vec of u8 and converts to a String
fn bytes_to_string(chunk: TtyChunk) -> String {
    let vec_chunk = chunk.to_vec();
    std::str::from_utf8(&vec_chunk).unwrap().to_owned()
}
