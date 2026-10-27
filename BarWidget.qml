import QtQuick
import Quickshell
import qs.Commons
import qs.Ui

BarWidget {
    id: root

    moduleName: "io.github.cybercoretech.tokengrid"
    readonly property string barIcon: "󰓡"

    readonly property string launchScript:
        localPath(Qt.resolvedUrl("helpers/launch-tokengrid.sh"))

    function localPath(url) {
        var value = String(url || "")
        if (value.indexOf("file://") === 0)
            value = value.substring(7)
        return decodeURIComponent(value)
    }

    function launch() {
        Quickshell.execDetached(["bash", launchScript])
    }

    function open() {
        launch()
    }

    function close() {}

    function toggle() {
        launch()
    }

    implicitWidth: button.implicitWidth
    implicitHeight: button.implicitHeight

    WidgetButton {
        id: button

        anchors.fill: parent
        bar: root.bar
        text: root.barIcon
        tooltipText: "TokenGrid AI Token Monitor"

        onPressed: function(buttonCode) {
            if (buttonCode === Qt.LeftButton)
                root.launch()
        }
    }
}
