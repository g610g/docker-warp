use docker_api::{
    self,
    conn::TtyChunk,
    opts::{ContainerListOpts, LogsOpts},
    Docker, Result,
};
use futures::StreamExt;

pub struct WsDocker {
    docker: Docker,
}

impl WsDocker {
    pub async fn run(&self) -> () {
        let mut containers = vec![];
        let container_interface = self.docker.containers();
        let container_list_opts = self.build_container_opts().await;
        let logging_opt = self.build_logging_options();
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
        for container in containers {
            let logopts_clone = logging_opt.clone();
            tokio::spawn(async move {
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
        }
    }
    fn build_logging_options(&self) -> LogsOpts {
        LogsOpts::builder()
            .all()
            .follow(true)
            .stdout(true)
            .stderr(true)
            .timestamps(true)
            .build()
    }
    async fn build_container_opts(&self) -> ContainerListOpts {
        let builder = ContainerListOpts::builder();
        builder.all(true).build()
    }
    pub fn new(path: &str) -> Result<Self> {
        let docker = docker_api::Docker::new(path).unwrap();
        Ok(WsDocker { docker })
    }
}
//deref the TtyChunk into vec of u8 and converts to a String
fn bytes_to_string(chunk: TtyChunk) -> String {
    let vec_chunk = chunk.to_vec();
    std::str::from_utf8(&vec_chunk).unwrap().to_owned()
}
