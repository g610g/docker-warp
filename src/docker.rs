use docker_api::{self, models::ContainerSummary, opts::ContainerListOpts, Container, Containers};
pub async fn docker() {
    let docker = docker_api::Docker::unix("/var/run/docker.sock");

    let _ = docker.version().await.unwrap();
    let containers = docker.containers();
    let containers_list = give_me_container(&containers).await;
    println!("Container List: {:?}", containers_list);
}
async fn give_me_container(container_interface: &Containers) -> Vec<Container> {
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
