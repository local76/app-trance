rSaver Support and Troubleshooting

Step 1: Run the Doctor Command
If it detects missing directories, incorrect screensaver files, or out-of-sync registry settings, you can instruct rSaver to heal itself automatically by running:
rsav doctor --fix

Step 2: Check the Logs
rSaver logs all events, system metrics, and download status to a background log file. This file contains valuable context if the application crashed or if a download failed.
Log Location: %APPDATA%\rSaver\rSaver.log
How to open in PowerShell:
notepad "$env:APPDATA\rSaver\rSaver.log"

Step 3: Open an Issue
If the doctor tool did not resolve your issue and you found an error in the logs, please open an issue in the official repository.
File a Bug or Feature Request at the GitHub Issues page: https://github.com/tourian-dynamics/rSaver/issues
What to include:
  Your Windows version (such as Windows 11 23H2).
  The terminal environment you are using (such as PowerShell 7, Command Prompt, Windows Terminal).
  The relevant output or error logs from %APPDATA%\rSaver\rSaver.log.
  Steps to reproduce the bug.
