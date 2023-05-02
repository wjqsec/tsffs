use crate::{
    config::{InputConfig, OutputConfig},
    messages::{client::ClientMessage, module::ModuleMessage},
    state::State,
    stops::StopReason,
};
use anyhow::Result;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use simics_api::{AttrValue, ConfObject};


pub trait ConfuseState {
    /// Callback when the module's state is [`ConfuseModuleState::HalfInitialized`]. The
    /// input config comes from the client, and the output config is modified by each
    /// [`Component`], where the last component's output configuration is returned to
    /// the client containing any information it needs.
    ///
    /// Any one-time configuration (registering callbacks, etc) should be done here.
    fn on_initialize(
        &mut self,
        _confuse: *mut ConfObject,
        _input_config: &InputConfig,
        output_config: OutputConfig,
    ) -> Result<OutputConfig> {
        Ok(output_config)
    }

    /// Callback after executionr reaches the `Magic::Start` instruction for the first time. This
    /// callback is called before `on_ready`
    fn pre_first_run(&mut self, _confuse: *mut ConfObject) -> Result<()> {
        Ok(())
    }

    /// Callback when the module is ready to run, it has hit the first [`Magic`] instruction and
    /// can be started. State that needs to be restored should be restored during this callback.
    /// This callback will run on each iteration of the fuzzing loop
    fn on_ready(&mut self, _confuse: *mut ConfObject) -> Result<()> {
        Ok(())
    }

    /// Callback when the module is about to run, just before the simulation is continued. Any
    /// setup that needs to be done before every run should be done here (for example, resetting
    /// the timeout duration).
    fn on_run(&mut self, _confuse: *mut ConfObject) -> Result<()> {
        Ok(())
    }

    /// Callback after execution has stopped, with some reason. Any cleanup or reporting that
    /// needs to be done after each run should be done here.
    fn on_stopped(&mut self, _confuse: *mut ConfObject, _reason: StopReason) -> Result<()> {
        Ok(())
    }

    /// Callback when the module has ben signaled to exit by the client. Any one-time cleanup or
    /// reporting should be done here.
    fn on_exit(&mut self, _confuse: *mut ConfObject) -> Result<()> {
        Ok(())
    }
}

pub trait ConfuseInterface {
    fn on_start(&mut self) -> Result<()> {
        Ok(())
    }

    fn on_add_processor(&mut self, _processor: *mut AttrValue) -> Result<()> {
        Ok(())
    }

    fn on_add_fault(&mut self, _fault: i64) -> Result<()> {
        Ok(())
    }
}

/// Trait for disassemblers of various architectures to implement to permit branch
/// and compare tracing
pub trait TracerDisassembler {
    fn disassemble(&mut self, bytes: &[u8]) -> Result<()>;
    fn last_was_control_flow(&self) -> Result<bool>;
    fn last_was_call(&self) -> Result<bool>;
    fn last_was_ret(&self) -> Result<bool>;
    fn last_was_cmp(&self) -> Result<bool>;
}

pub trait ConfuseClient {
    /// Get a mutable reference to the internal client state
    fn state_mut(&mut self) -> &mut State;

    /// Get a mutable reference to the internal client message sender channel, tx
    fn tx_mut(&mut self) -> &mut IpcSender<ClientMessage>;

    /// Get a mutable reference to the internal module message receiver channel, rx
    fn rx_mut(&mut self) -> &mut IpcReceiver<ModuleMessage>;

    /// Initialize the client with a configuration. The client will return an output
    /// configuration which contains various information the SIMICS module needs to
    /// inform the client of, including memory maps for coverage. Changes the
    /// internal state from `Uninitialized` to `HalfInitialized` and then from
    /// `HalfInitialized` to `ConfuseModuleState::Initialized`.
    fn initialize(&mut self, config: InputConfig) -> Result<OutputConfig>;

    /// Reset the module to the beginning of the fuzz loop (the state as snapshotted).
    /// Changes the internal state from `Stopped` or `Initialized` to `HalfReady`, then
    /// from `HalfReady` to `Ready`.
    fn reset(&mut self) -> Result<()>;

    /// Signal the module to run the target software. Changes the intenal state from `Ready` to
    /// `Running`, then once the run finishes either with a normal stop, a timeout, or a crash,
    /// from `Running` to `Stopped`. This function blocks until the target software stops and the
    /// module detects it, so it may take a long time or if there is an unexpected bug it may
    /// hang.
    fn run(&mut self, input: Vec<u8>) -> Result<StopReason>;

    /// Signal the module to exit SIMICS, stopping the fuzzing process. Changes the internal state
    /// from any state to `Done`.
    fn exit(&mut self) -> Result<()>;

    /// Send a message to the module
    fn send_msg(&mut self, msg: ClientMessage) -> Result<()> {
        self.state_mut().consume(&msg)?;
        self.tx_mut().send(msg)?;
        Ok(())
    }

    /// Receive a message from the module
    fn recv_msg(&mut self) -> Result<ModuleMessage> {
        let msg = self.rx_mut().recv()?;
        self.state_mut().consume(&msg)?;
        Ok(msg)
    }
}
