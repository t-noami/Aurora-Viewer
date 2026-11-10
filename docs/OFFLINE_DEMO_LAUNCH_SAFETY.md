# Running an offline demo safely

During an October v0.8.3 investigation, inspecting the application by its mounted path opened a separate normal instance after the intended process had exited. The user saw the login screen. The demo-success claim and the 65.2 FPS figure from that session were withdrawn. Grid access and credential transmission were not established. [Incident record](archive/metal-2026-10/plans/OFFLINE_DEMO_LAUNCH_SAFETY.md)

1. Start the intended executable as an owned child with the supported demo environment and a unique test title. For example, on macOS use `AURORA_DEMO=1 cargo run -p aurora-viewer -- --title "Test metal-v083"`. Record the process identity and allowlisted demo settings.
2. Before inspecting a capture or collecting measurements, confirm that the child is alive and its log has a positive demo-start marker. Use the viewer's own capture from that run. Looking up an app by name or path in a launch-capable tool can start another instance.
3. If the child exits, a login window appears, or the marker is missing, stop and discard the sample. Leave unrelated processes alone. Keep login screens, account data and grid locations out of captures and reports. Only the human performs real-grid checks.

The [testing guide](TESTING.md) lists the capture options.
