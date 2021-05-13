use futures_util::stream::StreamExt;
use tokio_tungstenite::connect_async;

const HOSTNAME: &'static str = "127.0.0.1";
const PORT: i32 = 5757;
const PID_KEY: &'static str = "pID";
const SDK_VERSION_KEY: &'static str = "sdkVersion";
const FLAVOR_KEY: &'static str = "sdkLanguage";
const FLAVOR: &'static str = "Rust";
const HEALTHCHECK_TIMEOUT_SECONDS: i32 = 60;

pub struct ServerState {
    is_network_initialized: bool,
    sender: Option<crate::aux_proxy_message_sender::AuxProxyMessageSender>,
    process_parameters: crate::process_parameters::ProcessParameters,
    is_process_ready: bool,
    game_session_id: Option<crate::entity::GameSessionId>,
    termination_time: i64,
}

impl ServerState {
    pub async fn initialize_networking(&mut self) -> Result<(), crate::error::GameLiftErrorType> {
        if !self.is_network_initialized {
            let (ws_stream, _) = connect_async(Self::create_uri())
                .await
                .expect("Failed to connect to GameLift Aux Proxy");

            let (write, read) = ws_stream.split();

            self.sender = Some(crate::aux_proxy_message_sender::AuxProxyMessageSender::new(
                write,
            ));

            self.is_network_initialized = true;
        }

        Ok(())
    }

    fn create_uri() -> String {
        format!("http://{}:{}", HOSTNAME, PORT)
    }

    pub async fn process_ready(
        &mut self,
        process_parameters: crate::process_parameters::ProcessParameters,
    ) -> Result<(), crate::error::GameLiftErrorType> {
        self.is_process_ready = true;
        self.process_parameters = process_parameters;

        if !self.is_network_initialized {
            return Err(crate::error::GameLiftErrorType::NetworkNotInitialized);
        }

        Ok(())
    }
}

/*public async ProcessReady(procParameters: ProcessParameters): Promise<GenericOutcome> {
this.processReady = true
this.processParameters = procParameters

if (!ServerState.networkInitialized) {
return new GenericOutcome(new GameLiftError(GameLiftErrorType.NetworkNotInitialized))
}

const result: GenericOutcome = await this.sender!.ProcessReady(
this.processParameters.Port,
this.processParameters.LogParameters.LogPaths
)

this.StartHealthCheck()

return result
}*/

/*public InitializeNetworking(): GenericOutcome {
if (!ServerState.networkInitialized) {
const socketToAuxProxy = io.connect(this.CreateURI(), this.CreateDefaultOptions())
const socketFromAuxProxy = io.connect(this.CreateURI(), this.CreateDefaultOptions())
this.sender = new AuxProxyMessageSender(socketToAuxProxy)
this.network = new Network(socketToAuxProxy, socketFromAuxProxy, this)
const outcome = this.network.Connect()
ServerState.networkInitialized = outcome.Success
return outcome
}

// Idempotent
return new GenericOutcome()
}*/

/*class ServerState implements IAuxProxyMessageHandler {
static networkInitialized: boolean = false
static readonly instance: ServerState = new ServerState()
static readonly debug = getDebug('ServerState')

sender?: AuxProxyMessageSender
network?: Network

processParameters?: ProcessParameters
processReady: boolean = false
gameSessionId?: string
terminationTime: Long = Long.NEG_ONE*/
