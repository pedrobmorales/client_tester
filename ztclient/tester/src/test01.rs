#[cfg(test)]
mod tests {
    use std::time;

    use bollard::Docker;
    use dotenv::dotenv;

    use tokio::time::sleep;

    use ztclient_common::containers::remove_container;
    use ztclient_common::kafka::{kafka_consumer, kafka_consumer_own};
    use ztclient_common::models::MachineUpdateMessage;
    use ztclient_common::ninjapanda::delete_machine;
    use ztclient_common::ztclient::{ztclient_execute, ztclient_logout, ztclient_status};
    use ztclient_common::{
        execute_callback, get_running_json, ExecuteCallbackRequest, RuntimeInformation,
    };
    use ztclient_common::{
        ztclient::{start_ztclientd, ztclient_registration},
        Config,
    };

    fn setup() -> (Docker, Config, RuntimeInformation) {
        // Initialize Docker client
        let docker = Docker::connect_with_unix_defaults().unwrap();
        dotenv().ok();

        let config = match envy::from_env::<Config>() {
            Ok(config) => config,
            Err(error) => panic!("{:#?}", error),
        };

        let runtime_information = get_running_json().unwrap();
        (docker, config, runtime_information)
    }

    async fn kafka_test() {
        let (docker, config, _runtime_information) = setup();

        let consumer = kafka_consumer(&docker, &config.kafka_container_name, "machine.register", 5)
            .await
            .unwrap();
        let json_objects: Vec<serde_json::Value> = consumer
            .iter()
            .map(|x| serde_json::from_str(x.as_str()))
            .filter(Result::is_ok)
            .map(Result::unwrap)
            .collect();
        dbg!(&json_objects, json_objects.len());
    }

    struct CustomContext;

    use rdkafka::client::ClientContext;
    use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
    use rdkafka::consumer::stream_consumer::StreamConsumer;
    use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
    use rdkafka::error::KafkaResult;
    use rdkafka::message::{Headers, Message};
    use rdkafka::topic_partition_list::TopicPartitionList;
    use rdkafka::util::get_rdkafka_version;

    impl ClientContext for CustomContext {}

    impl ConsumerContext for CustomContext {
        fn pre_rebalance(&self, rebalance: &Rebalance) {
            println!("Pre rebalance {:?}", rebalance);
        }

        fn post_rebalance(&self, rebalance: &Rebalance) {
            println!("Post rebalance {:?}", rebalance);
        }

        fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
            println!("Committing offsets: {:?}", result);
        }
    }

    async fn rdkafka_test01() {
        let context = CustomContext;
        // A type alias with your custom consumer can be created for convenience.
        type LoggingConsumer = StreamConsumer<CustomContext>;

        let consumer: LoggingConsumer = ClientConfig::new()
            .set("group.id", "my_group_id")
            .set("bootstrap.servers", "127.0.0.1:19094")
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            //.set("statistics.interval.ms", "30000")
            //.set("auto.offset.reset", "smallest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Consumer creation failed");

        consumer
            .subscribe(&["machine.update"])
            .expect("Can't subscribe to specified topics");

        loop {
            println!("WTF in this loop");
            match consumer.recv().await {
                Err(e) => log::warn!("Kafka error: {}", e),
                Ok(m) => {
                    let payload = match m.payload_view::<str>() {
                        None => "",
                        Some(Ok(s)) => s,
                        Some(Err(e)) => {
                            println!("Error while deserializing message payload: {:?}", e);
                            ""
                        }
                    };
                    println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                    if let Some(headers) = m.headers() {
                        for header in headers.iter() {
                            println!("  Header {:#?}: {:?}", header.key, header.value);
                        }
                    }
                    consumer.commit_message(&m, CommitMode::Async).unwrap();
                }
            };
        }
    }
}
