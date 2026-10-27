import QtQuick
import Quickshell
import Quickshell.Io
import qs.Ui

BarIconButton {
    id: root
    text: "󰓡"
    
    Process {
        id: tuiProc
        command: ["/home/raven/Devspace/Cybercore/ai/omarchy-plugins/tokengrid/bin/tokengrid-tui"]
    }

    onClicked: {
        tuiProc.running = true
    }
}
