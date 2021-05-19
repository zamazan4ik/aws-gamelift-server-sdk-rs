pub struct Network {
    socket_to_aux_proxy: rust_socketio::Socket,
    socket_from_aux_proxy: rust_socketio::Socket,
    is_connected: bool,
}

impl Network {
    pub fn new(
        socket_to_aux_proxy: rust_socketio::Socket,
        socket_from_aux_proxy: rust_socketio::Socket,
    ) -> Self {
        Self {
            socket_to_aux_proxy,
            socket_from_aux_proxy,
            is_connected: false,
        }
    }

    pub fn connect(&mut self) {}

    fn set_handler_callbacks(&mut self, socket: &mut rust_socketio::Socket) {}
}
