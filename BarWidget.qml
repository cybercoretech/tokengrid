import QtQuick
import Quickshell
import qs.Ui

BarIconButton {
    id: root
    text: "󰓡"
    onClicked: {
        bar.run("/home/raven/Devspace/Cybercore/ai/omarchy-plugins/tokengrid/bin/tokengrid-tui")
    }
}
