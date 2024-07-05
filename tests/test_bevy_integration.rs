//! A single integration test that we can send and receive serial data through the bevy plugin.
// Right now, this test can only run where we can reliably create files as placeholder ports, since
// we need a fixed targets of 2 ports for bidirectional communication. Many CI systems don't let
// you just connect to any random COM port.
#[cfg(target_os = "linux")]
#[test]
fn test_receive_bytes_send_through_serial_port_from_bevy_app() -> Result<(), String> {
    use bevy::prelude::{App, MinimalPlugins, PostStartup, Startup, Update};
    use bevy_serialport::SerialPortPlugin;
    use internal_nonsense::{run_in_background_with_deadline, with_local_serial_connected_ports};
    use receive_or_panic_bevy_app_impl::{
        poll_serial_messages_10_times_exit_app_if_found_else_panic, send_test_data, setup_receiver,
        setup_sender, TestPTTYPortNames,
    };
    // If there is a timeout, we probably aren't receiving the data and the app will hang.
    run_in_background_with_deadline(std::time::Duration::from_millis(2000), || {
        with_local_serial_connected_ports(|serial_port_name, serial_port_name2: String| {
            let mut app = App::new();
            // The app should only update for 10 ticks before exiting gracefully or panicing.
            // Typically calling .run() will never return, which is bad for a test and
            // CI.
            app.add_plugins((MinimalPlugins, SerialPortPlugin))
                .insert_resource(TestPTTYPortNames {
                    sender: String::from(serial_port_name),
                    receiver: String::from(serial_port_name2),
                })
                .add_systems(Startup, (setup_receiver, setup_sender))
                .add_systems(PostStartup, send_test_data)
                .add_systems(
                    Update,
                    poll_serial_messages_10_times_exit_app_if_found_else_panic,
                );
            app.run(); // This should shut down if we received a serial signal.
        })
    })
}

/// A simple bevy app that: sends data to a serial port, exits if that data is received within 10
/// update "ticks" , and panics otherwise.
#[cfg(target_os = "linux")]
mod receive_or_panic_bevy_app_impl {
    use bevy::{
        app::AppExit,
        prelude::{EventReader, EventWriter, Local, Res, ResMut, Resource},
        utils::tracing::info,
    };
    use bevy_serialport::{
        DataBits, FlowControl, Parity, SerialData, SerialPortRuntime, SerialPortSetting,
        SerialResource, StopBits,
    };
    use bytes::Bytes;
    use std::num::NonZero;
    #[derive(Debug, Resource)]
    pub(super) struct TestPTTYPortNames {
        pub sender: String,
        pub receiver: String,
    }
    /// Shutdown when we receive a response
    pub(super) fn poll_serial_messages_10_times_exit_app_if_found_else_panic(
        mut serial_ev: EventReader<SerialData>,
        mut shutdown_writer: EventWriter<AppExit>,
        port_names: Res<TestPTTYPortNames>,
        mut n_times_polled: Local<u8>,
    ) {
        *n_times_polled += 1;
        if *n_times_polled >= 10 {
            // should be enough for this test. A local ptty without wires should
            // be fast
            panic!("Failed to find a serial message after 10 polls. Are we receiving data");
        }
        for message in serial_ev.read().filter(|x| x.port == port_names.receiver) {
            info!("receive {:?}", message);
            // Exit the app gracefully to pass the test
            shutdown_writer.send(AppExit::Error(NonZero::new(100).unwrap()));
        }
    }

    pub(super) fn send_test_data(
        mut serial_res: ResMut<SerialResource>,
        port_name: Res<TestPTTYPortNames>,
    ) {
        serial_res.send_message(&port_name.sender, Bytes::from(&b"123457"[..]))
    }
    pub(super) fn setup_receiver(
        ports: Res<TestPTTYPortNames>,
        mut serial_res: ResMut<SerialResource>,
        rt: Res<SerialPortRuntime>,
    ) {
        serial_res
            .open(rt.clone(), &ports.receiver, 115_200)
            .expect(&format!(
                "Error opening serial port. {:?}. Available ports: {:?}",
                &ports,
                &serial_res.ports.keys()
            ));
    }
    pub(super) fn setup_sender(
        ports: Res<TestPTTYPortNames>,
        mut serial_res: ResMut<SerialResource>,
        rt: Res<SerialPortRuntime>,
    ) {
        let serial_setting = SerialPortSetting {
            port_name: ports.sender.clone(),
            baud_rate: 115_200,
            data_bits: DataBits::Five,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Default::default(),
        };
        serial_res
            .open_with_setting(rt.clone(), serial_setting)
            .expect(&format!(
                "Error opening serial port. {:?}. Available ports: {:?}",
                &ports,
                &serial_res.ports.keys()
            ));
    }
}

#[cfg(target_os = "linux")] // Whatever linux-specific hacks I need to get tests to work
mod internal_nonsense {
    use std::{panic::UnwindSafe, sync::mpsc, thread, time::Duration};
    use tempdir::TempDir;
    pub(super) type TimeoutError = String;
    pub(super) type TimeoutResult<T> = Result<T, TimeoutError>;
    // https://github.com/rust-lang/rfcs/issues/2798#issuecomment-552949300
    /// Run a function in a background thread and return an error if the execution time exceeds the
    /// provided deadline.
    pub(super) fn run_in_background_with_deadline<T, F>(d: Duration, f: F) -> TimeoutResult<T>
    where
        T: Send + 'static,
        F: FnOnce() -> T,
        F: Send + 'static,
    {
        let (done_tx, done_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let val = f();
            done_tx
                .send(())
                .map_err(|e| format!("Unable to send completion signal. {}", e))?;
            Ok(val)
        });

        match done_rx
            .recv_timeout(d)
            .map_err(|e| format!("The function timed out {:?}", e))
        {
            Ok(_) => match handle.join() {
                Ok(h) => h,
                Err(_) => Err(format!("Uncaught exception")),
            },
            Err(e) => Err(e),
        }
    }

    /// Execute a function that accepts names of 2 bidirectional ptys (serial data) (software
    /// approximation of hardware). This connection is destroyed even if the invoked function
    /// panics, to prevent a zombie process.
    pub(super) fn with_local_serial_connected_ports<F, T>(test_impl: F)
    where
        T: Send + 'static,
        F: FnOnce(String, String) -> T + UnwindSafe,
        F: Send + 'static,
    {
        let (port1, port2, mut child) = ephemeral_serially_linked_file_ptys();
        // Analog of try-catch/python context manager to close the "mocked" pty to prevent a zombie
        // process
        // For some reason, this doesn't work when the test fn is run in another thread. idk.
        let maybe_res = std::panic::catch_unwind(move || test_impl(port1, port2));
        child.kill().expect("Failed to kill the socat process");
        maybe_res.expect("Thread panicked");
    }
    /// Returns 2 file names that are linked ptys. Very linux specific
    /// (https://linux.die.net/man/1/socat).
    fn ephemeral_serially_linked_file_ptys() -> (String, String, std::process::Child) {
        let tmp_path = TempDir::new("bevy_serialport_test")
            .expect("Could not find a temporary directory for faked serial port communication")
            .into_path();
        let fname1 = tmp_path.join("file1");
        let fname2 = tmp_path.join("file2");
        std::fs::write(&fname1, []).expect("Failed to write an empty temp file");
        std::fs::write(&fname2, []).expect("Failed to write an empty temp file");
        let abs_fname1 = fname1
            .canonicalize()
            .expect("Could not find file for serial communication")
            .into_os_string()
            .into_string()
            .unwrap();
        let abs_fname2 = fname2
            .canonicalize()
            .expect("Could not find file for serial communication")
            .into_os_string()
            .into_string()
            .unwrap();

        // https://stackoverflow.com/questions/52187/virtual-serial-port-for-linux
        // socat -d -d pty,raw,echo=0,link=file1 pty,raw,echo=0,link=file2
        let mut socat_child = std::process::Command::new("socat")
            .args([
                "-d",
                "-d",
                &format!("pty,raw,echo=0,link={abs_fname1}"),
                &format!("pty,raw,echo=0,link={abs_fname2}"),
            ])
            .spawn()
            .expect("Failed to start a fake serial port");

        std::thread::sleep(std::time::Duration::from_millis(1000));
        match socat_child
            .try_wait()
            .expect("Failed to run the socat command to emulate serial comms")
        {
            None => {}
            Some(e) => panic!("socat process should not have exited. {:?}", e),
        }
        (abs_fname1, abs_fname2, socat_child)
    }
}
