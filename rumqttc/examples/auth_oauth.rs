use rumqttc::v5::mqttbytes::v5::AuthProperties;
use rumqttc::v5::{mqttbytes::QoS, AsyncClient, MqttOptions};
use rumqttc::{TlsConfiguration, Transport};
use std::error::Error;
use std::sync::Arc;
use tokio::task;
use tokio_rustls::rustls::ClientConfig;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let pubsub_access_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsIng1dCI6IkwxS2ZLRklfam5YYndXYzIyeFp4dzFzVUhIMCIsImtpZCI6IkwxS2ZLRklfam5YYndXYzIyeFp4dzFzVUhIMCJ9.eyJhdWQiOiJodHRwczovL2V2ZW50Z3JpZC5henVyZS5uZXQiLCJpc3MiOiJodHRwczovL3N0cy53aW5kb3dzLm5ldC84MWQ4MDMwMC1kNTEyLTQyNDItYjYwNC01ZTc0NDg3OWNjMmYvIiwiaWF0IjoxNzE2ODc5MTQ5LCJuYmYiOjE3MTY4NzkxNDksImV4cCI6MTcxNjg4MzA0OSwiYWlvIjoiRTJOZ1lOamkwYzBhL3Fqay91bU9ZMTQrdG0rekFRPT0iLCJhcHBpZCI6ImE1NzhmZjJmLTFjZTgtNGZhYy05ZWVkLTZiZjNlYWJjMzYyNyIsImFwcGlkYWNyIjoiMSIsImlkcCI6Imh0dHBzOi8vc3RzLndpbmRvd3MubmV0LzgxZDgwMzAwLWQ1MTItNDI0Mi1iNjA0LTVlNzQ0ODc5Y2MyZi8iLCJvaWQiOiJhY2JjOTMwMy0xYzBkLTRkMjItOTZmZC02ZmFlNzliMWVlMDgiLCJyaCI6IjAuQVQ0QUFBUFlnUkxWUWtLMkJGNTBTSG5NTDNnS1BJTGdYVVZFcF9YQzlDMTl5SnMtQUFBLiIsInN1YiI6ImFjYmM5MzAzLTFjMGQtNGQyMi05NmZkLTZmYWU3OWIxZWUwOCIsInRpZCI6IjgxZDgwMzAwLWQ1MTItNDI0Mi1iNjA0LTVlNzQ0ODc5Y2MyZiIsInV0aSI6ImZZdE1WSzZuT1VlY0hyNWdLRnV5QUEiLCJ2ZXIiOiIxLjAifQ.jl5WfEZJY5Bi9a1QSbtyocoCp9zv-4h9cKJMhO-hGioQODiIgehtYryKJxjlR1bSeg1W6TkzyIDDJGovKGg3YCofiJ5TF_8jBpM6ZiY7U8wbUndg5ziKvXHCPbhqGkfF3TMhDHnrPPZ3gO_YWO0S7SdomzNPcmzN1f14qCywEQKZ-bI8qMhMJDelMU3OP1s5JBrkuY-Mfh9X8QE4rd0gXYtzyurVu_LN0NM1BZs5ZDUqGQFsW9OW6EoRQvRA04XLYWVuRwYh1rRABLrjZGNFy1OOz7YJJljDMCL_qjPFEVi9k_4Gr4pTGBlr0GiVPXNlKulAV-Z8jFLDeDzzc9gKJg";

    let mut mqttoptions = MqttOptions::new(
        "client1-session1",
        "auth-test.eastus-1.ts.eventgrid.azure.net",
        8883,
    );
    mqttoptions.set_authentication_method(Some("OAUTH2-JWT".to_string()));
    mqttoptions.set_authentication_data(Some(pubsub_access_token.into()));

    // Use rustls-native-certs to load root certificates from the operating system.
    let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
    root_cert_store.add_parsable_certificates(
        rustls_native_certs::load_native_certs().expect("could not load platform certs"),
    );

    let client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let transport = Transport::Tls(TlsConfiguration::Rustls(Arc::new(client_config.into())));

    mqttoptions.set_transport(transport);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    task::spawn(async move {
        client.subscribe("topic1", QoS::AtLeastOnce).await.unwrap();
        client
            .publish("topic1", QoS::AtLeastOnce, false, "hello world")
            .await
            .unwrap();

        // Re-authentication test.
        let props = AuthProperties {
            authentication_method: Some("OAUTH2-JWT".to_string()),
            authentication_data: Some(pubsub_access_token.into()),
            reason_string: None,
            user_properties: Vec::new(),
        };

        client.reauth(Some(props)).await.unwrap();
    });

    loop {
        let notification = eventloop.poll().await;

        match notification {
            Ok(event) => println!("{:?}", event),
            Err(e) => {
                println!("Error = {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
