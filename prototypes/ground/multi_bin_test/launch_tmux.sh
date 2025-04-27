SESSION_NAME="multi_rust_apps"

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
TARGET_DIR="$SCRIPT_DIR/target/debug"


if ! command -v tmux &> /dev/null; then
    echo "Error: tmux is not installed."
    echo "Please install tmux to use this script (e.g., 'sudo apt install tmux', 'sudo dnf install tmux')."
    exit 1
fi


APP1_PATH="$TARGET_DIR/app1"
APP2_PATH="$TARGET_DIR/app2"
APP3_PATH="$TARGET_DIR/app3"

if [ ! -f "$APP1_PATH" ] || [ ! -f "$APP2_PATH" ] || [ ! -f "$APP3_PATH" ]; then
  echo "Executables not found in $TARGET_DIR."
  echo "Expected paths:"
  echo "  $APP1_PATH"
  echo "  $APP2_PATH"
  echo "  $APP3_PATH"
  echo "Have you run 'cargo build' in the '$SCRIPT_DIR' directory?"
  exit 1
fi

# Session Management
if tmux has-session -t "$SESSION_NAME" 2>/dev/null; then
  echo "Killing existing tmux session '$SESSION_NAME'..."
  tmux kill-session -t "$SESSION_NAME"
fi

echo "Starting applications in tmux session '$SESSION_NAME'..."

RUN_CMD_APP1="cd '$TARGET_DIR' && ./app1; echo -e '\\nApp1 finished. Press Enter to close this pane.' ; read"
RUN_CMD_APP2="cd '$TARGET_DIR' && ./app2; echo -e '\\nApp2 finished. Press Enter to close this pane.' ; read"
RUN_CMD_APP3="cd '$TARGET_DIR' && ./app3; echo -e '\\nApp3 finished. Press Enter to close this pane.' ; read"

tmux new-session -d -s "$SESSION_NAME" -n "App1" "$RUN_CMD_APP1"
tmux new-window -t "$SESSION_NAME" -n "App2" "$RUN_CMD_APP2"
tmux new-window -t "$SESSION_NAME" -n "App3" "$RUN_CMD_APP3"

tmux select-window -t "$SESSION_NAME:0"

echo "Tmux session '$SESSION_NAME' created successfully."

