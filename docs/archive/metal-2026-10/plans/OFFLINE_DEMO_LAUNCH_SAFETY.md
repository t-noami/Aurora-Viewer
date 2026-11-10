# Offline demo launch incident and prevention

Date: 30 October 2026. Applies to all automated Aurora Viewer demo testing in this project.

## Incident and evidence boundary

A demo-only task resulted in an unauthorized normal viewer launch and display of the login screen. Calling the computer-use app accessor by the mounted application's path could launch the application after the intended child process had exited. The normal instance did not inherit the explicitly supplied demo environment. The agent then inspected the wrong window. This crossed the user's authorized execution boundary and is treated as a security incident, not just a measurement mistake.

The user reports that the demo was not launched and normal mode was shown. Earlier process logs do not establish that the displayed window was the intended demo. Withdraw the prior demo-success claim and quarantine its FPS values pending a fresh, positively identified run. No grid connection or credential transmission has been established by this investigation; do not assert absence of impact without evidence. Do not inspect credentials or real account storage to investigate it.

## Mandatory prevention

- Never use `getApp` by app name/path, LaunchServices, Finder, `open`, or an equivalent launch-capable UI selector to inspect a demo. Never retry a missing/exited demo by launching the app normally.
- Launch only the exact reviewed executable as an owned child with explicit demo environment, unique test title, isolated test output and bounded lifetime. Record only allowlisted non-secret launch settings; never dump the full environment.
- Use the official command `AURORA_DEMO=1 cargo run -p aurora-viewer -- --title "Test metal-v083"`. No custom launcher, startup redesign or proof of credential-store isolation is required before this supported demo procedure. Do not access credentials.
- Confirm the owned PID is still alive and has a positive demo startup marker before any UI interaction or measurement. A title alone is insufficient. Use only an existing non-launching surface if independently supported; otherwise inspect the application's own demo capture with a file image viewer.
- If the process exits, the marker is missing, an unexpected window appears or login UI is observed: stop automation and discard the sample. Do not inspect the login screen, click login, read fields, take further screenshots, or activate/relaunch the app. Terminate only a child process positively owned by this run; do not kill unrelated user processes.
- Do not reuse or redistribute the accidental login screenshot, account details or location. Store only sanitized incident facts. Do not silently delete user artifacts.
- Correction: the earlier requirement to implement a guarded launcher and stub tests was excessive and is withdrawn. Prevent the actual failure by avoiding any separate launch-capable UI accessor.
- Real-grid checks and normal startup are performed by the human. A demo request never authorizes normal mode.

## Status

Rules recorded and included in the repair handoff. Use the official demo launch immediately for the authorized offline work; verify the resulting scene and measurements. Do not delay graphics repair for a new launch framework.
