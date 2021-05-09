mod aws {
    mod gamelift {
        #[allow(dead_code)]
        pub enum GameliftErrorType {
            AlreadyInitialized, // The GameLift Server or Client has already been initialized with Initialize().
            FleetMismatch, // The target fleet does not match the fleet of a gameSession or playerSession.
            GameliftClientNotInitialized, // The GameLift client has not been initialized.
            GameliftServerNotInitialized, // The GameLift server has not been initialized.
            GameSessionEndedFailed, // The GameLift Server SDK could not contact the service to report the game session ended.
            GameSessionNotReady,    // The GameLift Server Game Session was not activated.
            GameSessionReadyFailed, // The GameLift Server SDK could not contact the service to report the game session is ready.
            InitializationMismatch, // A client method was called after Server::Initialize(), or vice versa.
            NotInitialized, // The GameLift Server or Client has not been initialized with Initialize().
            NoTargetAliasidSet, // A target aliasId has not been set.
            NoTargetFleetSet, // A target fleet has not been set.
            ProcessEndingFailed, // The GameLift Server SDK could not contact the service to report the process is ending.
            ProcessNotActive, // The server process is not yet active, not bound to a GameSession, and cannot accept or process PlayerSessions.
            ProcessNotReady,  // The server process is not yet ready to be activated.
            ProcessReadyFailed, // The GameLift Server SDK could not contact the service to report the process is ready.
            SdkVersionDetectionFailed, // SDK version detection failed.
            ServiceCallFailed,  // A call to an AWS service has failed.
            StxCallFailed,      // A call to the XStx server backend component has failed.
            StxInitializationFailed, // The XStx server backend component has failed to initialize.
            UnexpectedPlayerSession, // An unregistered player session was encountered by the server.
            BadRequestException,
            InternalServiceException,
        }
    }
}
