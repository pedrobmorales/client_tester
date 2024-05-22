use std::{thread::sleep, time::Duration};

use bollard::{container::RemoveContainerOptions, Docker};

pub struct ContainerRemover {
    pub container_name: String,
}

impl ContainerRemover {
    pub fn new(container_name: String) -> ContainerRemover {
        ContainerRemover { container_name }
    }
}

impl Drop for ContainerRemover {
    fn drop(&mut self) {
        println!("Dropping container!!");
        let docker = Docker::connect_with_local_defaults().unwrap();
        let name = self.container_name.clone();
        let res = tokio::spawn(async move {
            println!("Inside dropping closure");
            remove_container(&docker, &name).await;
        });
        sleep(Duration::from_secs(1));
    }
}
pub async fn remove_container(docker: &Docker, container_name: &str) {
    let debug_containers = std::env::var("TEST_DEBUG_CONTAINERS").is_ok();
    if debug_containers {
        return;
    }
    let remove_result = docker
        .remove_container(
            container_name,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await;
    if remove_result.is_err() {}
}
