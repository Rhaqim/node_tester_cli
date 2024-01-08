/// The `cascade_api` module contains functions for initializing and running the CLI.
pub mod cascade_api {
    use crate::config::Config;
    use crate::core::{CliArgs, InitArgs};
    use crate::report::{Report, ReportData, ReportHeader};
    use crate::service::http_web3;
    use crate::{error, info};
    use web3::transports::Http;
    use web3::types::BlockNumber;
    use web3::Transport;
    use web3::Web3;

    /// Initializes the CLI with the provided arguments.
    ///
    /// Uses the `Config` struct to save the node address.
    ///
    /// # Arguments
    ///
    /// * `args` - The initialization arguments.
    pub async fn initialise_cli(args: InitArgs) {
        let mut config = Config::new();

        config.node_address = args.node.clone();
        config.save();
    }

    /// Tests the connection to the saved node.
    ///  
    /// # Arguments
    ///     
    /// * `args` - The CLI arguments.
    ///     
    /// # Returns
    ///
    /// * `true` if the connection is successful, `false` otherwise.
    ///    
    /// # Panics
    ///
    /// * If the node is not a websocket node.
    /// * If the node is not a HTTP node.
    pub async fn test_cli_node(args: CliArgs) {
        let node = Config::load().node_address;

        if node.is_empty() {
            error!("Node address not initialized. Use 'init' command to set the node.");
            return;
        }

        // initialize the http transport
        let web3s = http_web3(node.clone());

        run_default_test(&web3s, args).await;
    }

    /// Runs the CLI with the provided arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - The CLI arguments.
    pub async fn run(args: CliArgs) {
        let node = Config::load().node_address;

        if node.is_empty() {
            error!("Node address not initialized. Use 'init' command to set the node.");
            return;
        }

        // initialize the http transport
        let web3s = http_web3(node.clone());

        run_with_query_http(&web3s, args).await;
    }

    /// Runs the default test when the address is not provided.
    ///
    /// # Arguments
    ///
    /// * `web3_http` - The HTTP Web3 instance.
    /// * `args` - The CLI arguments.
    async fn run_default_test(web3_http: &Web3<Http>, args: CliArgs) {
        let from_block = BlockNumber::Number(args.from.into());
        let to_block = BlockNumber::Number(args.to.into());

        let logs = web3_http
            .eth()
            .logs(
                web3::types::FilterBuilder::default()
                    .from_block(from_block)
                    .to_block(to_block)
                    .build(),
            )
            .await
            .expect("failed to fetch logs");

        if args.method == "logs" {
            info!("Logs length: {:?}", logs.len());
        } else {
            run_with_query_http(&web3_http, args).await;
        }
    }

    /// Runs the HTTP test with query parameters.
    ///
    /// # Arguments
    ///
    /// * `web3_http` - The HTTP Web3 instance.
    /// * `args` - The CLI arguments.
    async fn run_with_query_http(web3_http: &Web3<Http>, args: CliArgs) {
        let transport = web3_http.transport();

        let header = ReportHeader {
            node: "node".to_string(),
            args: args.clone(),
        };

        print!("Running with query parameters: {:?}", args.params);

        // let params_serde = vec![helpers::serialize(&args.params)];
        let params_serde = vec![serde_json::json!({
            "fromBlock": args.from,
            "toBlock": args.to,
            "address": format!("0x{}", args.address),
        })];

        let get_logs = transport.execute(&args.method, params_serde).await;

        match get_logs {
            Ok(_) => {
                let data = [ReportData {
                    success: true,
                    error: None,
                    duration: 0,
                    result: Some("Success".to_string()),
                }]
                .to_vec();

                let mut report = Report::new(header);

                report.add_data(data[0].clone());

                report.display();
            }
            Err(e) => {
                error!("Error: {:?}", e);
            }
        }
    }
}
