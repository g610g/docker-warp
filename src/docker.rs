use docker_api::{
    self,
    conn::TtyChunk,
    opts::{ContainerListOpts, LogsOpts, ServiceListOpts},
    Container, Docker, Result,
};
#[derive(Clone)]
pub struct WsDocker {
    docker: Docker,
}
impl WsDocker {
    // pub async fn run_services(&self) {
    //     let service_interface = self.docker.services();
    //     let services = service_interface
    //         .list(&Self::build_service_opts())
    //         .await
    //         .unwrap();
    //     for service in services {
    //         if let Some(id) = service.id {
    //             let api_service = service_interface.get(id);
    //             while let Some(chunk) = api_service.logs(&self.build_logging_options()).next().await
    //             {
    //                 println!("{}", bytes_to_string(chunk.unwrap()));
    //             }
    //         }
    //     }
    // }
    // fn build_service_opts() -> ServiceListOpts {
    //     ServiceListOpts::builder().build()
    // }
    pub async fn give_containers(&self) -> Vec<Container> {
        let mut containers = vec![];
        let container_interface = self.docker.containers();
        let container_list_opts = self.build_container_opts().await;
        let _ = self.build_logging_options();
        let container_summaries = container_interface
            .list(&container_list_opts)
            .await
            .unwrap();
        for container_summary in container_summaries {
            match container_summary.id {
                Some(id) => containers.push(container_interface.get(id)),
                None => println!("No id for this container"),
            };
        }
        return containers;
    }
    pub fn build_logging_options(&self) -> LogsOpts {
        LogsOpts::builder()
            .all()
            .since(&chrono::offset::Local::now())
            .stdout(true)
            .timestamps(true)
            .build()
    }
    async fn build_container_opts(&self) -> ContainerListOpts {
        let builder = ContainerListOpts::builder();
        builder.all(true).build()
    }
    pub fn new(path: &str) -> Result<Self> {
        let docker = docker_api::Docker::unix(path);
        Ok(WsDocker { docker })
    }
}
//deref the TtyChunk into vec of u8 and converts to a String
pub fn bytes_to_string(chunk: TtyChunk) -> String {
    let vec_chunk = chunk.to_vec();
    std::str::from_utf8(&vec_chunk).unwrap().to_owned()
}
