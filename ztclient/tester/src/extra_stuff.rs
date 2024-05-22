async fn _example_read_logs(docker: &Docker, container_name: &str) {
    // let ten_millis = time::Duration::from_secs(5);
    // thread::sleep(ten_millis);

    println!("Starting log file collect");

    let mut output = docker.logs(
        container_name,
        Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,

            ..Default::default()
        }),
    );

    while let Some(item) = output.next().await {
        println!("Item: {}", item.unwrap());
    }
    println!("This isn't really expected to end");
}

/// This function is unused, but keeping it here as a reminder on how to read logs from a running container.
async fn _read_client_logs(docker: &Docker, container_name: &str) {
    // let ten_millis = time::Duration::from_secs(5);
    // thread::sleep(ten_millis);

    log::info!("Starting log file collect for {container_name}");

    let now = SystemTime::now();
    let _now_time = now.duration_since(UNIX_EPOCH).expect("Of course");

    for _x in 1..3 {
        let mut output = docker.logs(
            container_name,
            Some(LogsOptions::<String> {
                stdout: true,
                stderr: true,
                follow: true,
                ..Default::default()
            }),
        );

        while let Some(item) = output.next().await {
            let line = item.unwrap();
            match line {
                LogOutput::StdErr { message } => {
                    println!("StdErr: {:?}", message);
                }
                LogOutput::StdOut { message } => {
                    println!("StdOut: {:?}", message);
                }
                LogOutput::StdIn { message } => {
                    println!("StdIn: {:?}", message);
                }
                LogOutput::Console { message } => {
                    println!("Console: {:?}", message);
                }
            }
        }
    }

    println!("This isn't really expected to end");
}
