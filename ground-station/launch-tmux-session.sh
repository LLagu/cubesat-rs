
EXECUTABLE="./target/debug/groundstation"

tmux new-session -d -s rust_split "$EXECUTABLE serial" \; \
     split-window -v -t rust_split \; \
     send-keys -t rust_split:.1 "$EXECUTABLE usb" C-m \; \
     select-pane -t rust_split:.0 \; \
     attach-session -t rust_split