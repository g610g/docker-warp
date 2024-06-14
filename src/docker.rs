use docker_api::{self, opts::ContainerListOpts, Containers};
pub async fn docker() {
    let docker = docker_api::Docker::unix("/var/run/docker.sock");

    let _ = docker.version().await.unwrap();
    let containers = docker.containers();
    // println!("Containers: {:?}", containers);
    list_containers(&containers).await;
}
async fn list_containers(container: &Containers) {
    let builder = ContainerListOpts::builder();
    let container_list_opts = builder.all(true).build();
    let container_list = container.list(&container_list_opts).await.unwrap();
    for container in container_list {
        match container.names {
            Some(names) => {
                println!("Name: {:?}", names);
            }
            None => {}
        }
    }
}
